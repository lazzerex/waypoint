# Architecture Decision Records

Short records of decisions that would be costly to reverse. Add new ones at the bottom. Never delete old ones. Mark them superseded instead.

Template:

```markdown
## ADR-NNN: Title
- Status: proposed | accepted | superseded by ADR-XXX
- Date: YYYY-MM-DD
- Context: what forces are at play
- Decision: what we chose
- Consequences: what gets easier, what gets harder
```

---

## ADR-001: Use Topcoat as the web framework

- **Status:** accepted
- **Date:** 2026-11-24
- **Context:** We want a Rust full-stack framework with server-rendered async components, client reactivity without a WASM build, sessions, mail, and OpenAPI generation. Topcoat provides these but is experimental and pre-1.0.
- **Decision:** Build the UI and HTTP layer on Topcoat. Pin an exact version and upgrade deliberately.
- **Consequences:**
  - Components query data directly, so no separate API layer is needed for the UI.
  - Framework churn is a real cost, mitigated by ADR-006.
  - Missing pieces (jobs, rate limiting, validation, user authentication) are ours to build.

## ADR-002: Use Toasty as the ORM

- **Status:** proposed
- **Date:** 2026-11-24
- **Context:** Toasty comes from the same team, and Topcoat's recommended preset scaffolds a Toasty/SQLite app with migration commands. It is young, and Topcoat lists better Toasty integration (form-to-record mapping) as future work.
- **Decision:** Use Toasty behind repository traits in `infra/db`. Map form and API input to models explicitly. Allow raw SQL for queries Toasty cannot express (job claiming, search).
- **Consequences:** Swapping the ORM is contained to one module. If Toasty blocks a critical feature, fall back to `sqlx` for that repository.
- **Revisit when:** a required query, driver, or migration workflow is unsupported.

## ADR-003: SQLite first, PostgreSQL for scale

- **Status:** proposed
- **Date:** 2026-11-24
- **Context:** The target users are small teams who value easy self-hosting. SQLite means zero operational dependencies and matches Topcoat's recommended preset. PostgreSQL becomes necessary for multiple instances. **Verify** Toasty's PostgreSQL support before committing.
- **Decision:** Develop and ship on SQLite. Keep SQL portable and isolate dialect-specific code (job claiming, search) behind traits so PostgreSQL support can follow in Phase 5.
- **Consequences:** Single-instance deployments initially. Live updates can use an in-process event stream. Avoid PostgreSQL-only features in shared code paths.

## ADR-004: `workspace_id` on every tenant table from day one

- **Status:** accepted
- **Date:** 2026-11-24
- **Context:** The MVP serves one team, but retrofitting multi-tenancy later is painful and risky.
- **Decision:** All tenant-owned tables carry `workspace_id`. Repository functions require a `WorkspaceId`. The MVP runs a single workspace.
- **Consequences:** Slight extra boilerplate now. A hosted multi-tenant offering later needs no schema rewrite.

## ADR-005: Inbound email via provider webhook, not IMAP polling

- **Status:** accepted
- **Date:** 2026-11-24
- **Context:** Topcoat's mail support is outbound only. IMAP polling adds connection management, state tracking, and latency. Providers (Postmark, Mailgun, SES) deliver parsed messages over HTTP.
- **Decision:** Accept inbound mail through an authenticated webhook that enqueues a processing job. Make processing idempotent on `Message-ID`.
- **Consequences:** Requires a provider account and MX/DNS setup, so self-hosters cannot just point at an existing mailbox. An IMAP adapter can be added later behind the same job.

## ADR-006: Keep domain and services independent of Topcoat

- **Status:** accepted
- **Date:** 2026-11-24
- **Context:** Topcoat is pre-1.0 and its API will change.
- **Decision:** `domain` has no I/O and no framework or ORM imports. `services` depend on infra traits. Only `app`, `api`, and `infra` touch Topcoat and Toasty types.
- **Consequences:** More mapping code between layers. Framework upgrades touch a small, well-defined surface.

## ADR-007: DB-backed job queue for background work

- **Status:** accepted (revisit if Topcoat ships jobs)
- **Date:** 2026-11-24
- **Context:** Email sending, webhook delivery, and inbound processing must be retryable and survive restarts. Topcoat lists background jobs as planned, and its docs advise starting durable work outside render paths.
- **Decision:** Implement a `jobs` table with atomic claiming (`UPDATE ... RETURNING` on SQLite, `FOR UPDATE SKIP LOCKED` on PostgreSQL) run by in-process tokio workers. Enqueue jobs in the same transaction as the state change.
- **Consequences:** No extra infrastructure such as Redis. Transactional enqueueing avoids lost or phantom jobs. Throughput is bounded by the database, which is fine at this scale.

## ADR-008: API types are separate from domain types

- **Status:** accepted
- **Date:** 2026-11-24
- **Context:** The public API must stay stable while internals evolve. Topcoat derives OpenAPI schemas from the types used in handlers via `schemars`.
- **Decision:** Define request and response structs in `src/app/api/v1` and derive `JsonSchema` there. Map to and from domain types explicitly.
- **Consequences:** Some duplication, but API changes are deliberate and reviewable through the generated OpenAPI diff in CI.

## ADR-009: Tickets are addressed by per-workspace number externally

- **Status:** accepted
- **Date:** 2026-11-24
- **Context:** Humans quote ticket numbers in emails and chat. UUIDs are unfriendly.
- **Decision:** Expose `number` in the UI, API, and email subjects (`[#1042]`). Keep UUIDs internal and for non-ticket resources.
- **Consequences:** Numbers are guessable, so every access check is workspace-scoped, and email thread matching requires the signed token or matching headers, never the number alone.

## ADR-010: Topcoat sessions with our own credential and session tables

- **Status:** accepted
- **Date:** 2026-11-24
- **Context:** Topcoat provides a bring-your-own-storage session lifecycle (start, refresh, rotate, stop) and secure cookie defaults, but no user accounts or login.
- **Decision:** Use Topcoat's session API and cookie defaults. Store only the token hash, user, and expiry in our `sessions` table. Implement Argon2id credentials in `services/auth.rs`. Guard with helper functions called from layers and from every shard and procedure.
- **Consequences:** We write less security-critical code than a fully custom scheme. We must remember that shards and procedures bypass page guards, so authorization is enforced in services as well.

## ADR-011: Live updates via connected renders and an event stream

- **Status:** proposed
- **Date:** 2026-11-24
- **Context:** Agents need fresh ticket lists without manual refresh. Topcoat's `live!`/`emit!` and `runtime::connected(cx)` stream re-rendered HTML over a WebSocket, restarting on reconnect.
- **Decision:** Publish `ticket_events` to a per-workspace in-process broadcast channel. Connected components re-render on relevant events and always reread state from the database. Use shards for filter and search interactions.
- **Consequences:** Simple and fast on a single instance. Scaling out needs a cross-instance source (event polling or `LISTEN/NOTIFY`). The per-connection render cap needs tuning.
- **Revisit when:** moving to multiple instances, or if the cap makes busy inboxes awkward.

## ADR-012: Tailwind and Topcoat UI for the interface

- **Status:** accepted
- **Date:** 2026-11-24
- **Context:** We want a professional-looking UI quickly and the freedom to customize it. Topcoat has built-in Tailwind support (no Node) and a shadcn-style library whose components are copied into the project.
- **Decision:** Use Tailwind and Topcoat UI. Add components with `topcoat ui add`, own and edit them in `src/components`, and commit `components.toml`.
- **Consequences:** No dependency on a design system we cannot change. Upstream component updates are merged manually. Class names must be literals so Tailwind can find them.
