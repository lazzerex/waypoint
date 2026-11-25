# Design decisions

A record of choices that shape the codebase and would be expensive to reverse.
Each entry states the context, the decision, and what it costs. New entries are
added at the bottom; superseded entries stay in place and point to their
replacement.

## Topcoat for the web layer

**Status:** accepted

Topcoat provides server-rendered async components, module-based routing,
sessions, mail, and OpenAPI generation in one framework. It is pre-1.0, so the
version is pinned and upgraded deliberately. The framework handles the request
and rendering model; user accounts, background jobs, validation, and rate
limiting remain application code.

## Toasty for persistence

**Status:** accepted

Toasty comes from the same team as Topcoat and supports SQLite and PostgreSQL.
It is young, so models stay behind repository functions in `infra`. Queries that
Toasty cannot express, such as atomic job claiming, use raw SQL. Replacing the
ORM would be contained to that layer.

## SQLite first, PostgreSQL ready

**Status:** accepted

Small teams should be able to deploy with a single file and no database server.
SQLite is the default and the supported deployment for a single instance.
PostgreSQL becomes relevant when more than one application instance is needed.
Shared code avoids dialect-specific features, and the few places that need them
are isolated.

## Workspace identifier on tenant data

**Status:** accepted

Every tenant-owned table carries a `workspace_id`, and repository functions
require one. The product currently runs a single workspace, but the schema does
not need to change if hosting multiple workspaces is offered later.

## Inbound email through a provider webhook

**Status:** accepted

Topcoat mail is outbound only. Accepting mail through a provider webhook avoids
the connection management and latency of IMAP polling, and providers already
deliver parsed messages over HTTP. The trade-off is that a deployment needs a
provider account and DNS configuration rather than an existing mailbox. An IMAP
adapter could be added behind the same processing job later.

## Framework-free domain layer

**Status:** accepted

`domain` holds types and business rules with no I/O and no framework or ORM
imports. `services` depend on domain types and infrastructure interfaces. Only
the web, API, and infrastructure layers mention Topcoat or Toasty. This keeps a
pre-1.0 framework upgrade away from business rules, at the cost of some mapping
code between layers.

## Database-backed job queue

**Status:** accepted

Sending mail, processing inbound messages, and delivering webhooks must survive
a restart and be retried. A `jobs` table claimed atomically by in-process
workers avoids adding Redis or another service. Jobs are inserted in the same
transaction as the state change that caused them, so a committed change always
has its follow-up work queued. Throughput is bounded by the database, which is
sufficient for the target team size.

## API types separate from domain types

**Status:** accepted

The public API must remain stable while internals change. Request and response
types live with the API routes and are mapped explicitly to domain types. This
duplicates some definitions, but it makes the public contract visible and lets
the generated OpenAPI document show any breaking change as a reviewable diff.

## Ticket numbers as the external identifier

**Status:** accepted

People quote ticket numbers in email and chat, so the per-workspace number is
what the interface, API, and email subjects use. UUIDs stay internal. Numbers
are guessable, so every lookup is scoped to the workspace, and email threading
relies on the signed reply token or mail headers rather than the number alone.

## Sessions with application-owned credentials

**Status:** accepted

Topcoat provides cookies and a session lifecycle but no user accounts. Waypoint
stores Argon2id password hashes and a session table that holds only a hash of
the cookie token, the user, and an expiry. Login rotates the token. Shards and
procedures have their own endpoints, so authorization is also enforced in
services rather than only in page or layout guards.

## Live updates from an event stream

**Status:** accepted

Agents should see new and changed tickets without refreshing. Ticket events are
published to a per-workspace channel, and connected components re-render and
reread current state from the database. This is simple on a single instance.
Multiple instances would need a shared event source, such as PostgreSQL
`LISTEN/NOTIFY` or polling the event table.

## Tailwind and Topcoat UI

**Status:** accepted

Tailwind is built without Node, and Topcoat UI components are copied into the
repository rather than consumed as a package. The team owns the markup and can
change it freely. `components.toml` records what was installed, so upstream
updates are merged on purpose. Tailwind class names must be written literally
in source for the scanner to find them.
