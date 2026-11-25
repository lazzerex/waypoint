<h1 align="center">Waypoint</h1>

<p align="center">
  <strong>A shared inbox and helpdesk for small teams.</strong><br />
  Customers reach you by web form or email. Every conversation becomes a ticket
  with an owner, a status, and a threaded history. Staff triage, assign, and reply
  from a fast server-rendered interface, with a JSON API and signed webhooks for
  everything else.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-2024-CE412B?logo=rust&logoColor=white" alt="Rust 2024" />
  <img src="https://img.shields.io/badge/Topcoat-0.12.1-4B9CD3" alt="Topcoat 0.12.1" />
  <img src="https://img.shields.io/badge/Toasty-0.11.0-333333" alt="Toasty 0.11.0" />
  <img src="https://img.shields.io/badge/SQLite-first-003B57?logo=sqlite&logoColor=white" alt="SQLite first" />
  <img src="https://img.shields.io/badge/PostgreSQL-ready-4169E1?logo=postgresql&logoColor=white" alt="PostgreSQL ready" />
  <img src="https://img.shields.io/badge/Tailwind-CSS-06B6D4?logo=tailwindcss&logoColor=white" alt="Tailwind CSS" />
</p>

<p align="center">
  <a href="https://github.com/lazzerex/waypoint/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/lazzerex/waypoint/ci.yml?label=CI&logo=githubactions&logoColor=white" alt="CI status" /></a>
  <img src="https://img.shields.io/badge/license-MIT-green" alt="MIT license" />
  <img src="https://img.shields.io/badge/status-early%20development-orange" alt="Status: early development" />
</p>

<p align="center">
  <a href="#why-waypoint">Why Waypoint</a> ·
  <a href="#core-workflows">Core workflows</a> ·
  <a href="#tech-stack">Tech stack</a> ·
  <a href="#getting-started">Getting started</a> ·
  <a href="#configuration">Configuration</a> ·
  <a href="#project-layout">Project layout</a> ·
  <a href="#documentation">Documentation</a>
</p>

---

## Why Waypoint

Small teams outgrow a shared Gmail inbox. It has no ownership, no status, and no
reliable history once more than one person starts replying. Enterprise helpdesks
solve that with a stack of services and a price to match.

Waypoint aims for the middle: tickets, assignment, canned replies, email
threading, and an API, deployed as one binary plus a database. It is built for
teams of two to twenty people who want to self-host and keep the operational
surface small.

The project is also a practical exercise in Topcoat's server-rendered component
model: async pages, forms, sessions, mail, live regions, and a generated OpenAPI
document in one Rust application.

## Core workflows

**Shared inbox**

- A public ticket form and inbound email both create tickets.
- Inbound replies attach to the existing ticket through a signed reply address
  and standard `In-Reply-To` and `References` headers.
- Auto-replies, bounces, and duplicate provider deliveries are filtered out.

**Triage**

- Ticket lists with status, assignee, and priority filters.
- "My tickets" and "Unassigned" views.
- Assignment, status changes, priority changes, and internal notes.
- A full activity history for every ticket.

**Conversation**

- A threaded view of customer messages, agent replies, and internal notes.
- Replies sent from the support address in the same email thread.
- Quoted-text stripping, HTML sanitization, and attachment storage.

**Team and integrations**

- Staff accounts with `admin` and `agent` roles.
- A documented JSON API, idempotency keys, and signed webhooks.
- OpenAPI generated from the same routes that serve traffic.

## Tech stack

| Layer | Technology | Role |
|---|---|---|
| Language | **Rust** (2024) | One compiled binary, no runtime |
| Web | **Topcoat** 0.12.1 | Module routing, async components, forms, sessions, live regions |
| UI | **Tailwind CSS** + **Topcoat UI** | Utility styling and editable components owned by the repository |
| Data | **Toasty** 0.11.0 | Models, queries, and migrations |
| Database | **SQLite** first, **PostgreSQL** ready | Zero-dependency local deployments; PostgreSQL for multiple instances |
| Outbound mail | **Topcoat mail** | SMTP in production, `.eml` files in development, in-memory in tests |
| Inbound mail | Provider webhook | Postmark, Mailgun, SES, or another HTTP delivery provider |
| API | **OpenAPI** + `schemars` | Spec generated from the router and API types |

Rationale for each choice is recorded in [DECISIONS.md](docs/DECISIONS.md).

## Getting started

### Prerequisites

- Rust stable toolchain
- The Topcoat CLI: `cargo install topcoat-cli`
- Optional: PostgreSQL 15+ (SQLite works out of the box)

### Run locally

```bash
git clone https://github.com/lazzerex/waypoint.git
cd waypoint

cp .env.example .env                         # edit values, see Configuration
cargo run -- toasty migration apply          # create or upgrade the database
topcoat dev                                  # build, bundle assets, serve with reload
```

The development server listens on `127.0.0.1:3000` by default. Override it with
`HOST` and `PORT`. In development, outbound mail is written to `.eml` files
instead of being sent.

### Common commands

```bash
topcoat dev                                     # development server
cargo test                                      # unit and integration tests
cargo clippy --all-targets -- -D warnings       # lints
cargo fmt --all                                 # Rust formatting
topcoat fmt --rustfmt                           # format view! and other macro bodies
cargo run -- toasty migration generate          # after changing models
cargo run -- toasty migration apply
topcoat ui add <component>                      # copy a Topcoat UI component
```

## Configuration

Environment variables.

| Variable | Required | Description |
|---|---|---|
| `DATABASE_URL` | yes | Database connection string (SQLite path or PostgreSQL URL) |
| `APP_BASE_URL` | yes | Public URL used in emails, webhook links, and absolute links |
| `SECRET_KEY` | yes | 32+ random bytes, base64. Signs cookies and reply-to tokens |
| `SUPPORT_DOMAIN` | yes | Domain for inbound mail, for example `support.example.com` |
| `MAIL_FROM` | yes | Default From address for outbound mail |
| `SMTP_URL` | production | SMTP connection URL. Unset in development to use the file transport |
| `INBOUND_EMAIL_SECRET` | yes | Shared secret for the inbound email webhook |
| `STORAGE_DIR` | no | Attachment storage path (default `./data/attachments`) |
| `TRUSTED_PROXIES` | production | Proxies allowed to set forwarding headers |
| `HOST`, `PORT` | no | Listen address (default `127.0.0.1:3000`) |
| `RUST_LOG` | no | Log filter, for example `info,waypoint=debug` |

## Project layout

```text
waypoint/
├── Cargo.toml
├── build.rs                 # Tailwind and icon build configuration
├── components.toml          # Topcoat UI manifest
├── styles.css               # Tailwind theme and source scanning
├── README.md
├── CONTRIBUTING.md
├── docs/
│   ├── PRODUCT.md           # vision, personas, scope, user stories
│   ├── ARCHITECTURE.md      # layers, module map, email/auth/live flows, security
│   ├── DATA_MODEL.md        # entities, relations, indexes, state machine
│   ├── API.md               # REST API, webhooks, OpenAPI
│   ├── ROADMAP.md           # product direction and engineering plan
│   └── DECISIONS.md         # architecture decision records
├── src/
│   ├── app/                 # routes, pages, layouts: public and staff areas
│   ├── components/          # UI components owned by the repository
│   ├── domain/              # pure types and business rules
│   ├── services/            # use cases and transactions
│   └── infra/               # database, mail, storage, search, jobs
├── tests/fixtures/emails/   # real .eml samples for threading tests
└── toasty/                  # generated migrations and snapshots
```

Layer dependencies point downward: `domain` has no I/O, `services` depend on
domain and infrastructure traits, and only `app`, `api`, and `infra` touch
Topcoat or Toasty types. See [ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Documentation

- [Product](docs/PRODUCT.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Data model](docs/DATA_MODEL.md)
- [API and webhooks](docs/API.md)
- [Roadmap](docs/ROADMAP.md)
- [Decision records](docs/DECISIONS.md)
- [Contributing](CONTRIBUTING.md)

## License

MIT. See [LICENSE](LICENSE).
