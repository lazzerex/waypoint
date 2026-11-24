# Waypoint

> A fast, self-hostable shared inbox and helpdesk for small teams, built in Rust with [Topcoat](https://github.com/tokio-rs/topcoat).

**Status:** pre-alpha / planning. Topcoat is itself experimental (0.x), so expect churn on both sides.

Customers reach you through a web form or by emailing a support address. Every conversation becomes a ticket with an owner, a status, and a threaded history. Staff triage, assign, and reply from a server-rendered UI. Everything is also available through a documented JSON API and signed webhooks.

## Why this exists

Small teams (2-20 people) outgrow a shared Gmail inbox, which has no ownership or status, but do not want a heavy enterprise helpdesk. Waypoint aims for the middle: tickets, assignment, canned replies, and an API, deployed as one binary plus a database.

It is also a realistic project for exercising Topcoat: async server components, forms, sessions, mail, live-updating views, and a generated OpenAPI spec.

## Features

### MVP (Phase 1)

- Public ticket form and inbound email (via provider webhook)
- Email threading: customer replies attach to the right ticket
- Ticket list with status, assignee, and priority filters
- Ticket view with conversation thread, internal notes, and activity history
- Assign, change status, change priority, reply by email
- Staff accounts with `admin` and `agent` roles
- Single workspace (schema is multi-tenant ready)

### Planned

See [docs/ROADMAP.md](docs/ROADMAP.md). Highlights: live-updating lists, canned replies, tags, search, JSON API, webhooks, SLAs, automation rules, reporting, customer portal.

## Tech stack

| Concern | Choice |
|---|---|
| Language | Rust (stable) |
| Web framework | [Topcoat](https://github.com/tokio-rs/topcoat): async server-rendered components, `$(...)` client reactivity, shards, `live!` regions |
| UI | Tailwind (Topcoat's built-in integration) + [Topcoat UI](https://github.com/tokio-rs/topcoat/blob/main/docs/ui.md) components copied into the repo |
| Database | SQLite first, PostgreSQL for multi-instance deployments |
| ORM | [Toasty](https://github.com/tokio-rs/toasty) |
| Sessions and cookies | Topcoat `session` and `cookie` modules, with our own credential and session tables |
| Outbound email | Topcoat `mail` (SMTP transport; file and in-memory transports for dev and tests) |
| Inbound email | Provider webhook (Postmark, Mailgun, SES, ...) |
| API docs | OpenAPI generated from routes (Topcoat `openapi` feature + `schemars`) |

Rationale for each choice is in [docs/DECISIONS.md](docs/DECISIONS.md).

## Getting started

### Prerequisites

- Rust stable toolchain
- The Topcoat CLI: `cargo install topcoat-cli`
- Optional: PostgreSQL 15+ (SQLite works out of the box)

### Run locally

```bash
git clone <your-repo-url> waypoint
cd waypoint

cp .env.example .env                         # edit values, see Configuration
cargo run -- toasty migration apply          # create/upgrade the database
topcoat dev                                  # build, bundle assets, serve with reload
```

The dev server listens on `127.0.0.1:3000` by default (override with `HOST` and `PORT`). In dev, outbound mail is written to `.eml` files by Topcoat's file transport instead of being sent, so you can open them in any mail client.

### Common commands

```bash
topcoat dev                                  # dev server
cargo test                                   # unit and integration tests
cargo clippy --all-targets -- -D warnings
cargo fmt --all
topcoat fmt --rustfmt                        # format view! and other macro bodies
cargo run -- toasty migration generate       # after changing models
cargo run -- toasty migration apply
topcoat ui add <component>                   # copy a Topcoat UI component into src/components
```

## Configuration

Environment variables.

| Variable | Required | Description |
|---|---|---|
| `DATABASE_URL` | yes | Database connection string (SQLite path or PostgreSQL URL) |
| `APP_BASE_URL` | yes | Public URL, used in emails, webhook links, and absolute `href!`s |
| `SECRET_KEY` | yes | 32+ random bytes (base64). Keys signed/private cookies and reply-to tokens |
| `SUPPORT_DOMAIN` | yes | Domain for inbound mail, e.g. `support.example.com` |
| `MAIL_FROM` | yes | Default From address for outbound mail |
| `SMTP_URL` | prod | SMTP connection URL. Unset in dev to use the file transport |
| `INBOUND_EMAIL_SECRET` | yes | Shared secret authenticating the inbound email webhook |
| `STORAGE_DIR` | no | Attachment storage path (default `./data/attachments`) |
| `TRUSTED_PROXIES` | prod | Proxy addresses allowed to set forwarding headers (needed for correct client IPs) |
| `HOST`, `PORT` | no | Listen address (default `127.0.0.1:3000`) |
| `RUST_LOG` | no | Log filter, e.g. `info,waypoint=debug` |

## Project layout

```text
waypoint/
├── Cargo.toml
├── build.rs              # Tailwind build config
├── components.toml       # Topcoat UI manifest (commit it)
├── README.md
├── CONTRIBUTING.md
├── docs/
│   ├── PRODUCT.md        # vision, personas, scope, user stories
│   ├── ARCHITECTURE.md   # layers, module map, email/auth/live flows, security
│   ├── DATA_MODEL.md     # entities, relations, indexes, state machine
│   ├── API.md            # REST API, webhooks, OpenAPI
│   ├── ROADMAP.md        # phased plan with checklists
│   └── DECISIONS.md      # architecture decision records
├── toasty/               # generated by Toasty (migrations and snapshots)
├── src/                  # see docs/ARCHITECTURE.md for the module map
└── tests/
    └── fixtures/emails/  # real .eml samples for threading tests
```

## Documentation

- [Product](docs/PRODUCT.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Data model](docs/DATA_MODEL.md)
- [API and webhooks](docs/API.md)
- [Roadmap](docs/ROADMAP.md)
- [Decision records](docs/DECISIONS.md)
- [Contributing](CONTRIBUTING.md)

## License

TBD (MIT or Apache-2.0 suggested).
