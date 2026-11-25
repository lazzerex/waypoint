# Roadmap

Status legend: `[ ]` not started, `[~]` in progress, `[x]` done.

Phases are ordered by value, not by date. Each phase should end with something usable.

## Phase 0: Foundation

Goal: a running app with auth, database, and CI.

- [ ] Scaffold with `topcoat new waypoint --recommended` (Toasty/SQLite, Tailwind), pin Topcoat and Toasty versions
- [ ] Config loading from environment
- [ ] Schema and migrations via Toasty: workspaces, users, sessions
- [ ] Root layout, `_public` and `_staff` group modules, base Topcoat UI components (`topcoat ui add`)
- [ ] Login / logout: Argon2id, Topcoat `session::start`/`stop`/`rotate`, persisted `token_hash`
- [ ] `current_actor(cx)` memoized helper and `require_actor(cx)` guard
- [ ] Seed command to create the first workspace and admin
- [ ] Logging with `tracing`, `/healthz`, `/readyz`
- [ ] CI: build, test, clippy, fmt, `topcoat fmt --check`
- [ ] Dockerfile that ships the binary and bundled `assets/`

**Exit criteria:** an admin can log in to an empty dashboard on a deployed instance.

## Phase 1: MVP, tickets and email

Goal: a team can run support from it.

### Tickets

- [ ] Schema: customers, mailboxes, tickets, messages, ticket_events
- [ ] Ticket number allocation per workspace
- [ ] Public ticket form with honeypot and rate limiting
- [ ] Ticket list with status/assignee/priority filters (a shard for filtering) and cursor pagination
- [ ] "My tickets" and "Unassigned" views
- [ ] Ticket detail with thread and activity history
- [ ] Assign, status, and priority changes (domain state machine + events)
- [ ] Internal notes

### Email

- [ ] DB-backed job queue and worker loop
- [ ] Outbound: replies and new-ticket confirmation via Topcoat `mail` (SMTP in prod, file transport in dev, in-memory in tests)
- [ ] Reply-To signed token and threading headers
- [ ] Inbound webhook endpoint, secret auth, raised `BodyLimit`, `process_inbound_email` job
- [ ] Threading by token and `In-Reply-To`/`References`
- [ ] Auto-reply and bounce detection, loop prevention
- [ ] Quoted-text stripping and HTML sanitization
- [ ] Attachments: receive, store, download, limits
- [ ] `.eml` fixture suite (Gmail, Outlook, Apple Mail, mobile clients)

### Team

- [ ] Invite and deactivate users, roles
- [ ] Mailbox settings (name, address, signature)

**Exit criteria:** you can run your own support for two weeks without losing a message.

## Phase 2: Productivity

- [ ] Live-updating ticket list and open ticket (`live!` + `connected(cx)` + event stream)
- [ ] Canned replies with shortcuts
- [ ] Tags
- [ ] Search (`SearchIndex` trait: FTS5 / `tsvector`)
- [ ] Keyboard shortcuts (next/prev ticket, reply, assign to me)
- [ ] Merge duplicate tickets
- [ ] Customer profile page with ticket history
- [ ] Notification preferences (email on assignment or mention)
- [ ] @mentions in internal notes
- [ ] Saved filters / views

## Phase 3: API and integrations

- [ ] API keys with scopes, `/api` bearer-auth layer
- [ ] REST API v1 (tickets, messages, customers)
- [ ] OpenAPI served at `/openapi.json`, CI diff check
- [ ] Webhooks with signing, retries, delivery log
- [ ] Idempotency keys and API rate limiting
- [ ] API docs page in the app

## Phase 4: SLAs, automation, reporting

- [ ] Business hours configuration
- [ ] SLA policies (first response, resolution) and breach tracking
- [ ] Due/overdue indicators in lists
- [ ] Automation rules (trigger, conditions, actions): auto-assign, auto-tag, auto-reply
- [ ] Round-robin assignment
- [ ] Reports: volume, first response time, resolution time, per-agent load
- [ ] CSV export

## Phase 5: Beyond the inbox

- [ ] PostgreSQL support and cross-instance live updates (`LISTEN/NOTIFY` or event polling)
- [ ] Customer portal (view own tickets via magic link)
- [ ] Localization of the public form and customer emails (Topcoat `l10n`, Fluent)
- [ ] Satisfaction (CSAT) survey on resolve
- [ ] Knowledge base with suggested articles on the public form
- [ ] Multiple mailboxes and brand-specific forms
- [ ] SSO (OIDC)
- [ ] Multi-workspace signup and billing, if offered as a hosted product
- [ ] Integrations: Slack notifications, Linear/GitHub issue linking

## Parking lot

- Chat widget
- "Collision detection" (two agents replying at once)
- Sender-domain rules
- Attachment virus scanning
- Data retention and GDPR export/delete tooling
- AI-assisted reply drafts and summaries
- Sitemap and public status pages (Topcoat has a `sitemap` feature)

## Topcoat capability check

Status as of Topcoat 0.12.x, from its README and `llms.txt`. Update this table when you bump versions.

| Need | Phase | Status |
|---|---|---|
| Module-based routing, layouts, layers | 0 | Available |
| Cookies (signed, private, prefixed) | 0 | Available |
| Session lifecycle (start, refresh, rotate, stop) | 0 | Available, bring your own storage |
| Login, users, password hashing | 0 | **Not provided.** Build in `services/auth.rs` ("Authentication" is on Topcoat's roadmap) |
| Cross-origin protection | 0 | `OriginPolicy` by default. **Verify** it covers form flows, otherwise add CSRF tokens |
| Tailwind, Topcoat UI, assets, fonts, icons | 0 | Available |
| Mail: SMTP, file, in-memory transports | 1 | Available (outbound only). **Verify** custom `Message-ID` and header control |
| Inbound email | 1 | **Not provided.** Provider webhook |
| Multipart and JSON extractors, body limits | 1 | Available |
| Rate limiting | 1 | **Not provided.** Tower layer (`tower` feature) or small custom layer |
| Validation helpers | 1 | **Not provided.** Validate in services/domain |
| Form-to-record mapping with Toasty | 1 | **Weak.** Map fields explicitly |
| Background jobs | 1 | **Not provided.** DB-backed queue |
| Shards, procedures, signals | 1-2 | Available. Must authorize inside each |
| Live updates (`live!`, `connected`, SSE, WebSocket) | 2 | Available. Check `max_runs_per_connection` |
| OpenAPI generation | 3 | Available (`openapi` feature) |
| Localization (Fluent) | 5 | Available (`l10n` feature) |
| Deployment guide | any | Not yet documented upstream. See [ARCHITECTURE.md](ARCHITECTURE.md#deployment) |
