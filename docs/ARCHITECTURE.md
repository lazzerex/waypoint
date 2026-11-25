# Architecture

> Topcoat is pre-1.0 and changes quickly. Framework-specific statements here come from Topcoat's README and `llms.txt` at the time of writing. Items marked **verify** were not confirmed. Check them against the version pinned in `Cargo.toml` and Topcoat's [docs](https://docs.rs/topcoat).

## Goals

- Keep business rules independent of the web framework and ORM, so framework churn is cheap.
- Make email handling reliable: idempotent, retryable, observable.
- Single deployable binary plus a database.

## Layers

```text
┌──────────────────────────────────────────────┐
│ app / api     Topcoat pages, routes, views    │  HTTP in, HTML/JSON out
├──────────────────────────────────────────────┤
│ services      Use cases, transactions         │  Orchestrates domain + infra
├──────────────────────────────────────────────┤
│ domain        Types, rules, state machines    │  Pure Rust, no I/O
├──────────────────────────────────────────────┤
│ infra         DB (Toasty), mail, storage, jobs│  Adapters behind traits
└──────────────────────────────────────────────┘
```

Dependency rule: arrows point downward only. `domain` depends on nothing. `services` depend on `domain` and on infra **traits**. `app`/`api` depend on `services`. Only `app`, `api`, and `infra` may use Topcoat or Toasty types.

## Module map

Topcoat's module-based routing derives URLs from the module tree (`module_router!()` maps its enclosing module to `/`; child modules add kebab-case segments; `_group` modules scope layouts and layers without adding a URL segment).

```text
src/
├── main.rs                 # config, wiring, start(module_router!()...), start workers
├── config.rs
├── app.rs                  # root layout (<html>, metadata, runtime script)
├── app/
│   ├── login.rs            # GET/POST /login
│   ├── logout.rs           # POST /logout
│   ├── _public.rs          # group: minimal layout, no auth
│   ├── _public/
│   │   └── support.rs      # /support  (public ticket form)
│   ├── _staff.rs           # group: shell layout + auth layer
│   ├── _staff/
│   │   ├── tickets.rs      # /tickets  (list, filters)
│   │   ├── tickets/
│   │   │   └── ticket_number.rs   # /tickets/{ticket_number} via module_param!
│   │   ├── customers.rs
│   │   └── settings.rs     # users, mailboxes, API keys, webhooks
│   ├── api.rs              # layer for /api: bearer auth, OpenAPI defaults
│   ├── api/v1/             # REST handlers
│   └── webhooks.rs         # inbound email endpoint, hidden from OpenAPI
├── components/             # Topcoat UI components (copied in) + app components
├── domain/
│   ├── ticket.rs           # Ticket, Status, Priority, transitions
│   ├── message.rs          # Message, MessageKind
│   ├── customer.rs
│   ├── user.rs             # Role, permissions
│   ├── reply_token.rs      # signed reply-to token logic
│   └── event.rs            # TicketEvent kinds
├── services/
│   ├── tickets.rs          # create, assign, change status, list
│   ├── messages.rs         # reply, note
│   ├── inbound_email.rs    # parse -> match thread -> create/append
│   ├── outbound_email.rs
│   ├── auth.rs             # credentials, session persistence, API keys
│   └── webhooks.rs         # (Phase 3)
└── infra/
    ├── db/                 # Toasty models + repositories
    ├── mail/               # inbound payload parsers, outbound builder
    ├── storage.rs          # attachment storage trait + filesystem impl
    ├── search.rs           # SearchIndex trait (FTS5 / tsvector)
    └── jobs.rs             # DB-backed queue + worker loop
```

Routes with absolute paths (e.g. `#[route(POST "/webhooks/inbound/email")]`) are registered with `.discover()` or `.route(handler)` rather than module derivation.

Start as a single crate. If boundaries or compile times demand it, split into a Cargo workspace along the layer lines (`waypoint-domain`, `waypoint-services`, `waypoint-web`).

## Request flow

1. Request reaches a Topcoat page or route. Layers run outermost first (least specific path first).
2. The `_staff` layer (web) or `/api` layer (API) resolves the caller into an `Actor { user_id, workspace_id, role }`.
3. The handler (or component) calls a service with the `Actor` and validated input.
4. The service opens a transaction, applies domain rules, persists via repositories, and writes a `ticket_events` row.
5. Side effects (emails, webhooks) are **enqueued as jobs in the same transaction**, never sent inline.
6. The handler renders a view, or returns JSON. After successful form posts it redirects with `see_other(href!(...).resolve(cx))` (POST/Redirect/GET).

Use `#[memoize]` helpers such as `current_actor(cx)` so the session lookup happens once per request no matter how many components ask.

## Authentication

Topcoat provides cookie handling and a session lifecycle API but not login, users, or password storage ("Authentication" is on its roadmap). We build the credential part:

- **Credentials:** `users.password_hash` (Argon2id). Login is a `POST` route that verifies the password, then calls `session::start(cx)`.
- **Session storage:** Topcoat issues the token; we persist `token_hash`, `user_id`, and expiry in the `sessions` table. We never store the raw token.
- **Lifecycle:** `session::token_hash(cx)` identifies the caller, and the app looks it up and validates expiry. `refresh` extends it (sliding expiration), `rotate` replaces the token (call on login and privilege changes, revoking the old hash), `stop` logs out (delete the returned hash).
- **Cookie:** default is `__Host-` prefixed, `Secure`, `HttpOnly`, `SameSite=Lax`, `Path=/`. Requires `.layer(CookieLayer::new())` and a saved key (`SECRET_KEY`) for signed/private cookies.
- **Guard:** the `_staff` layer redirects unauthenticated requests to `/login`. Prefer small guard helpers (`require_actor(cx)`) called wherever protection is needed.

> **Shards and procedures bypass page and layout guards.** Their endpoints are separate. Every shard and procedure must call `require_actor(cx)` itself, and services must re-check permissions with the `Actor`. Treat all their inputs as untrusted.

**API keys** use a different path: the `/api` layer reads `Authorization: Bearer`, hashes the key, looks it up, checks scopes, and builds the same `Actor`. No cookies are involved, so `OriginPolicy` is disabled for that path prefix only.

## Authorization

- Permission rules live in `domain::user` (e.g. `can_manage_users(role)`). Services call them. UI and API handlers do not make policy decisions.
- Repositories require a `WorkspaceId`. Wrap IDs in newtypes (`WorkspaceId`, `TicketId`) so a missing filter is hard to write.

## Email

Email is the hardest part of the product. Treat it as its own subsystem.

### Inbound

Topcoat's mail module is outbound only. Receive mail through a provider that POSTs it to us rather than polling IMAP.

```text
Provider ──POST──▶ /webhooks/inbound/email
                    │  verify shared secret (constant-time)
                    │  raise BodyLimit for this route (default buffered limit is 2 MiB)
                    │  persist raw payload (retention window, for replay)
                    │  enqueue job: process_inbound_email
                    └─▶ respond 200 quickly

Worker: process_inbound_email
  1. Parse payload: from, to, subject, text, html, headers, attachments
  2. Drop if automated: Auto-Submitted, Precedence: bulk, X-Autoreply, null return-path
  3. Idempotency: Message-ID already stored → stop
  4. Find ticket (first match wins):
       a. Signed token in recipient address:  help+t-1042.<sig>@support.example.com
       b. In-Reply-To / References matching a stored email_message_id
       c. none → new ticket
  5. Upsert customer by sender address
  6. Strip quoted reply text and signatures (keep the full original)
  7. Sanitize HTML, store attachments
  8. Create ticket or append message; apply status rules; write events
  9. Enqueue confirmation email (new tickets only)
```

Notes:

- Providers differ: Postmark posts JSON, others (Mailgun, SendGrid) post `multipart/form-data`. Use Topcoat's `Json` extractor or its `multipart` feature accordingly. `BodyLimit` applies across multipart fields.
- Do **not** thread by subject line. It leaks across customers.
- Respond to the provider fast and do the work in the job, so timeouts do not trigger duplicate deliveries.
- A token match is only accepted if the sender is the ticket's requester or a known participant. Ticket numbers are guessable, so never trust them alone.
- The webhook route authenticates with the shared secret, so disable `OriginPolicy` on `/webhooks` only.

### Outbound

Build mail with Topcoat's `Mail::builder()` and deliver via `Mailer` with an SMTP transport in production, the file transport (writes `.eml`) in dev, and the in-memory transport in tests. Email bodies can be written as `view!` templates through the builder's `html(cx, view! { ... })`.

- Agent replies create a `messages` row (`delivery_status = queued`) and a `send_email` job in one transaction.
- Headers on every outbound message (**verify** the builder supports setting these, especially `Message-ID`):
  - `Message-ID`: generated by us and stored on the message
  - `In-Reply-To` and `References`: from the thread
  - `Reply-To`: `help+t-<number>.<sig>@support.example.com`
  - `Auto-Submitted: auto-generated` on automated mail (confirmations)
- Retries with backoff. Permanent failures mark the message `failed` and show a banner on the ticket.

### Loop prevention

- Never auto-reply to automated senders.
- Rate-limit confirmations per sender address (e.g. max 3/hour).
- Never confirm mail that was identified as auto-submitted.

## Background jobs

Topcoat does not provide background jobs yet (it is on their roadmap), and its docs say to start durable jobs outside render paths. So:

- A `jobs` table (see [DATA_MODEL.md](DATA_MODEL.md#jobs)) is the queue. Jobs are enqueued in the same transaction as the state change.
- Workers are tokio tasks in the same binary. They claim work with an atomic `UPDATE ... RETURNING` (SQLite) or `SELECT ... FOR UPDATE SKIP LOCKED` (PostgreSQL), set `locked_at`, run, then mark `done` or reschedule with exponential backoff. The claim query is the one dialect-specific piece. Use raw SQL if Toasty cannot express it.
- Jobs must be idempotent. Include a stable idempotency key when the side effect is external.
- Stale locks (`locked_at` older than a timeout) are reclaimed.

If Topcoat ships a first-party job system, swap `infra/jobs.rs` behind the same trait.

## Live-updating UI

Topcoat gives us three relevant tools:

| Tool | Use in Waypoint |
|---|---|
| **Signals and `$(...)` expressions** | Instant client-only interactions: dropdowns, composer state, optimistic UI toggles |
| **Shards** (`#[shard]`) | Re-render a server component when its arguments change: filters, search-as-you-type on the ticket list |
| **`live!` / `emit!` + `runtime::connected(cx)`** | Push fresh HTML over the page's WebSocket when something changes |

Design for the ticket list and open ticket:

1. Initial HTTP render emits current state and finishes.
2. When `connected(cx)` is true, the component subscribes to a per-workspace event stream (an in-process `tokio::sync::broadcast` fed by `ticket_events` writes), and `emit!`s a re-rendered fragment on each relevant event.
3. On reconnect Topcoat restarts rendering, so the component rereads state from the database. It never relies on missed events.

Constraints to plan for:

- `RuntimeLayer::new().max_runs_per_connection(n)` caps connected renders per WebSocket (default 64). Raise it or design emissions accordingly. **Verify** behavior under a busy inbox.
- In-process broadcast only works on a single instance. For multiple instances, poll `ticket_events` by last-seen id, or use PostgreSQL `LISTEN/NOTIFY`.
- Use `#[key(item.id)]` on list items so Topcoat keeps identity across updates.
- Shards and procedures must authorize themselves (see [Authentication](#authentication)).

Alternatives if the built-in model is not enough: Topcoat's `sse` and `websocket` features, or its Datastar/htmx integrations.

## API and OpenAPI

- REST handlers live under `src/app/api/v1`, mounted at `/api/v1`.
- Enable the `openapi` feature and `schemars`. A layer on `/api` carries the shared tag, `401` response, and security scheme. Each route adds its own `Operation`.
- Serve the generated document at `/openapi.json`. CI regenerates it and diffs against a committed copy.
- API request and response types are separate from domain types.

Details in [API.md](API.md).

## Frontend

- **Styling:** Topcoat's Tailwind feature (no Node), configured in `build.rs`. It scans literal class names only, so never assemble class names dynamically.
- **Components:** Topcoat UI components are copied into `src/components` with `topcoat ui add`, so we own and can edit them. Commit `components.toml`.
- **Assets:** declared with `asset!`, bundled by `topcoat dev` and `topcoat asset bundle --release`, served with content-hashed URLs. Fonts and icons use the `font` and `icon` features.
- **Metadata:** page titles and descriptions via Topcoat's `Metadata` set with `with(...)` on layouts and pages.
- **Navigation:** `runtime::link` for client-side navigation without full reloads, with `PrefetchMode::Intent` by default.
- **i18n (later):** Topcoat's `l10n` feature (Fluent files) for the public form and customer-facing emails.

## Security

| Area | Approach |
|---|---|
| Passwords | Argon2id. Enforce minimum length, no composition rules |
| Sessions | Topcoat sessions: `__Host-`, `Secure`, `HttpOnly`, `SameSite=Lax`. Hash stored, rotate on login, sliding expiry |
| Cross-site requests | Topcoat's default `OriginPolicy` checks cross-origin mutations and WebSockets. All mutations are `POST`. Disable only on token-authenticated prefixes. **Verify** it fully covers our form flows, otherwise add a CSRF token |
| Shards and procedures | Authorize inside each one. Validate every input. Never capture secrets in `$(...)` |
| Public form | Honeypot, per-IP rate limit, size limits, optional CAPTCHA later. Rate limiting is not built into Topcoat yet (a Tower layer via its `tower` feature or a small custom layer works) |
| Client IP | `client_ip(cx)` ignores forwarding headers unless `.trusted_proxies(...)` is configured. Set it correctly behind a proxy or rate limits can be bypassed |
| HTML email | Allowlist sanitizer (e.g. `ammonia`) before storing `body_html`. Render in a restricted context |
| Attachments | Size and count limits, opaque storage keys, `Content-Disposition: attachment`, correct `Content-Type`, never inline-render untrusted types |
| Request bodies | Keep the default 2 MiB buffered limit except on the inbound email webhook and attachment upload routes |
| API keys | Shown once, stored hashed, scoped, revocable |
| Webhooks | HMAC-SHA256 with timestamp and replay window |
| Tenancy | `workspace_id` on every row and query |
| Secrets | Environment only. Never logged |
| Audit | `ticket_events` is append-only |

## Observability

- Structured logs with `tracing`, including `workspace_id`, `ticket_id`, and `job_id`.
- `/healthz` (process up) and `/readyz` (database reachable).
- Early metrics: inbound emails processed/failed, outbound emails queued/sent/failed, job queue depth and age, request latency.
- Retain raw inbound payloads for a window (e.g. 14 days) so failed parses can be replayed.

## Deployment

- Ship the compiled binary **and** its bundled `assets/` directory. Topcoat loads `assets/` beside the binary with `AssetBundle::load()` and panics on missing assets, so deploy them together. Build with `topcoat asset bundle --release` matching the cargo profile and package.
- SQLite deployments: one instance, database file on a persistent volume, regular backups (e.g. SQLite online backup or Litestream).
- PostgreSQL deployments: allow multiple instances. Attachments then need shared storage (S3-compatible), and live updates need a cross-instance event source.
- Run behind a reverse proxy with TLS and configure `trusted_proxies`.
- Run migrations on startup or via a separate command (`toasty migration apply`).
- Configuration through environment variables (see README).

## Performance notes

- The list query uses the `(workspace_id, status, last_activity_at desc)` index with cursor pagination.
- Avoid N+1 queries on the list: batch-load requesters and assignees.
- Search: SQLite FTS5 or PostgreSQL `tsvector` behind a `SearchIndex` trait (Phase 2).
- Topcoat renders components concurrently and outputs in source order. Do ordered side effects before constructing views.
- Keep fixed text and attributes literal inside `view!` and render numbers directly rather than with `format!`.

## Known risks

| Risk | Mitigation |
|---|---|
| Topcoat API churn | Pin version, confine to `app`/`api`/`infra`, keep domain pure |
| Toasty maturity (query coverage, migrations, form-to-record mapping is still on Topcoat's roadmap) | Thin repositories behind traits, explicit field mapping, raw SQL escape hatch |
| SQLite and PostgreSQL divergence | Keep SQL portable, isolate dialect-specific queries (job claiming, search) |
| Email edge cases (quoting, charsets, clients) | Fixture-driven tests with real `.eml` samples, replayable raw payloads |
| Live-update limits (`max_runs_per_connection`, single-instance broadcast) | Reread state on every render, plan a cross-instance source before scaling out |
| Missing built-ins (jobs, rate limiting, validation, higher-level auth) | Small in-repo implementations behind traits, replaced when Topcoat ships them |
