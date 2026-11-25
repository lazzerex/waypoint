# Architecture

Waypoint is one asynchronous Rust service. It serves the staff interface, the
public ticket form, the JSON API, and the inbound email webhook, and it runs its
own background workers. The only external dependency is a database.

## System shape

```text
Browser ──HTTP──▶ Topcoat router ──▶ pages / routes ──▶ services ──▶ domain
                                          │                │
                                          │                ├──▶ repositories ──▶ database
                                          │                └──▶ job queue ──▶ workers
                                          │
Email provider ──webhook──▶ inbound endpoint ──▶ job queue
                                                      │
                                                      ├──▶ SMTP / file / memory transport
                                                      └──▶ webhook delivery
```

The web layer never sends mail or calls an external service inline. A state
change and the jobs it implies are written in the same database transaction.
Workers pick up those jobs and perform the side effects later.

## Runtime stack

| Concern | Choice |
|---|---|
| Language | Rust 2024 |
| Web framework | Topcoat 0.12.1 |
| Templates and components | Topcoat `view!`, async components |
| Client behavior | Topcoat runtime, shards, procedures, live regions |
| Styling | Tailwind CSS, built without Node |
| UI components | Topcoat UI, copied into `src/components` |
| Persistence | Toasty 0.11.0 |
| Database | SQLite for single-instance deployments; PostgreSQL driver available |
| Background work | Database-backed job table with in-process workers |
| Mail | Topcoat mail with SMTP, file, and in-memory transports |
| API description | OpenAPI generated from the router |

## Layers

The code is arranged in four layers with a strict dependency direction.

**`app`** contains pages, layouts, routes, shards, procedures, and the HTTP API.
It translates requests into service calls and renders results. Topcoat types are
allowed here.

**`services`** contains use cases. A service receives an authenticated actor and
validated input, opens a transaction, applies domain rules, writes state, and
enqueues jobs. Services depend on domain types and infrastructure traits.

**`domain`** contains ticket, message, customer, user, event, and reply-token
types. It includes state transitions and permission rules but performs no I/O
and imports neither Topcoat nor Toasty.

**`infra`** adapts the outside world: Toasty models and repositories, mail
transports, attachment storage, search, and the job queue.

Only `app`, `api`, and `infra` may mention framework or ORM types. The boundary
keeps a framework upgrade from reaching into business rules.

## Route tree

Topcoat derives URLs from the module tree. A module named `_public` or `_staff`
is a group: it adds a layout or layer but no URL segment.

```text
src/
├── app.rs                    GET /
├── app/
│   ├── _public.rs            public layout
│   ├── _public/
│   │   └── support.rs        GET /support
│   ├── _staff.rs             staff shell and authentication boundary
│   └── _staff/
│       └── tickets.rs        GET /tickets
├── components/               shared UI components
├── domain/                   pure types and rules
├── services/                 application use cases
└── infra/                    database, mail, storage, search, jobs
```

Future ticket detail pages use a module parameter, for example
`_staff/tickets/{ticket_number}`, so the public ticket number remains the
identifier people see.

## Request lifecycle

1. A request reaches the router. Cross-origin checks run before layers.
2. The staff layer resolves the session cookie into an actor, or redirects to
   the login page. The API layer resolves a bearer key instead.
3. The page or route calls a service with the actor and validated input.
4. The service opens a transaction, applies domain rules, persists changes, and
   writes a `ticket_events` row.
5. Jobs for mail or webhooks are inserted in the same transaction.
6. The handler renders HTML or JSON. A successful form post redirects with a
   303 response so a refresh does not repeat the mutation.

Session lookups and other repeated per-request reads are memoized, so a layout,
page, and several components can ask for the current actor without repeating the
database work.

## Identity and access

Topcoat provides cookies and a session lifecycle, not user accounts. Waypoint
stores credentials and sessions itself.

- Passwords are hashed with Argon2id.
- A session cookie carries a random token. The database stores only its hash,
  the user, an expiry, and request metadata.
- Login rotates the token and revokes the previous hash. Logout deletes the
  current hash.
- The cookie is `__Host-` prefixed, `Secure`, `HttpOnly`, `SameSite=Lax`, and
  scoped to `/`.
- The staff boundary redirects unauthenticated page requests. Shards and
  procedures have their own endpoints and therefore perform the same check
  themselves.
- API keys are separate from sessions. The API layer hashes the bearer token,
  looks it up, checks scopes, and builds the same actor type.

Permission rules live in `domain`. Services enforce them; pages and API handlers
do not make policy decisions on their own.

## Mail

Mail is the most failure-prone part of a helpdesk, so it is treated as a
subsystem with explicit idempotency and retry rules.

### Inbound

An email provider posts parsed mail to `/webhooks/inbound/email`. The endpoint
checks a shared secret in constant time, stores the raw payload for replay, and
enqueues a processing job before returning quickly.

The worker then:

1. Parses sender, recipients, subject, bodies, headers, and attachments.
2. Drops automated mail, bounces, and bulk messages.
3. Stops if the `Message-ID` has already been stored.
4. Finds the ticket by signed reply token first, then by `In-Reply-To` and
   `References`.
5. Creates or updates the customer record.
6. Removes quoted text, sanitizes HTML, and stores attachments.
7. Appends the message and applies status rules.
8. Enqueues a confirmation when the message created a new ticket.

Subject-line matching is deliberately not used: it can merge unrelated
customers into one thread.

### Outbound

Agent replies are written as messages with a queued delivery status, together
with a `send_email` job. The builder sets our own `Message-ID`, the thread's
`In-Reply-To` and `References`, a signed `Reply-To`, and `Auto-Submitted` on
automated mail.

Production uses SMTP. Development writes `.eml` files. Tests use the in-memory
transport and assert on recipients, headers, and body.

### Loop prevention

Automated senders never receive an automatic reply. Confirmation mail is rate
limited per sender address, and any message marked as auto-submitted is excluded
from ticket creation.

## Background work

A `jobs` table is the queue. Claiming is a single atomic update; SQLite uses
`UPDATE ... RETURNING`, and PostgreSQL uses `SELECT ... FOR UPDATE SKIP LOCKED`.
A worker records `locked_at`, runs the job, then marks it done or schedules a
retry with exponential backoff. Stale locks are reclaimable.

Jobs must be idempotent because a worker can crash after the side effect but
before the status update. External calls carry a stable idempotency key where
the provider supports one.

## Real-time views

The ticket list and open ticket can refresh without a full page reload.

Topcoat's `live!` regions emit fresh HTML over the page's WebSocket. A
per-workspace broadcast channel is fed by ticket event writes. Connected
components subscribe, re-render, and always reread current state from the
database. A reconnect restarts rendering, so missed events do not leave stale
data.

The in-process channel works for a single instance. Multiple instances need a
cross-instance source such as PostgreSQL `LISTEN/NOTIFY` or polling the event
table by sequence.

## HTTP API

API handlers live under `src/app/api/v1` and share a layer that applies bearer
authentication and common OpenAPI settings. Request and response types are
separate from domain types so the public contract can stay stable while
internals change.

A route is documented only when it carries an operation. The generated document
is served at `/openapi.json` and checked in CI against a committed copy to catch
unintended breaking changes. See [api.md](api.md).

## Frontend

Tailwind scans literal class names in Rust source. Class names are never
assembled dynamically, because the scanner cannot see them.

Topcoat UI components are copied into the repository, not consumed as an opaque
package. The team owns their markup and styles and can edit them freely.
`components.toml` records what was installed so upstream changes can be merged
deliberately.

Assets are declared with `asset!`, hashed at build time, and loaded from the
asset bundle beside the binary. The binary and bundle must be deployed together.

## Deployment

A deployment consists of the compiled binary, its `assets/` directory, and a
database.

SQLite deployments run one instance and keep the database file on a persistent
volume with regular backups. PostgreSQL deployments can run several instances,
but then attachments need shared storage and live updates need a cross-instance
event source.

TLS terminates at a reverse proxy. The proxy address must be configured as a
trusted proxy so client IPs used by rate limits cannot be forged. Migrations run
at startup or through a separate command.

## Observability

Structured `tracing` logs carry workspace, ticket, and job identifiers. Health
endpoints distinguish process liveness from database readiness. Useful metrics
include inbound processing failures, outbound queue depth, job age, and request
latency. Raw inbound payloads are retained for a short replay window.

## Security posture

| Area | Approach |
|---|---|
| Passwords | Argon2id, minimum length, no composition rules |
| Sessions | Hashed tokens, rotation on login, sliding expiry, secure cookie defaults |
| Cross-site requests | Topcoat origin policy for browser mutations; POST for every mutation |
| Shards and procedures | Authenticate and validate inside each endpoint |
| Public form | Honeypot, per-IP rate limit, body size limits |
| HTML email | Allowlist sanitizer before storage |
| Attachments | Size and count limits, opaque keys, download disposition |
| API keys | Hashed at rest, scoped, revocable |
| Webhooks | HMAC-SHA256 with timestamp and replay window |
| Tenancy | `workspace_id` on every tenant row and query |
| Audit | Append-only ticket events |

Secrets come from the environment and are never logged or sent to the browser.
