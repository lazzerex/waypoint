# Roadmap

Waypoint is in early development. The foundation is in place: the application
boots, serves a public page and a staff area, and connects to a database. The
sections below describe where the product goes next, in the order the work
becomes useful.

## Near term: a working inbox

The first usable release lets a small team run support from Waypoint alone.

- Staff accounts with `admin` and `agent` roles, password login, and sessions.
- Mailbox settings for the support address and signature.
- A public ticket form that creates a ticket and sends a confirmation.
- A ticket queue with status, assignee, and priority filters.
- A ticket page with the full thread and an activity history.
- Assignment, status changes, priority changes, and internal notes.
- Outbound replies sent from the support address, in the same email thread.
- Inbound email through a provider webhook, matched to the right ticket.

The measure of this stage is simple: a team can run its support address through
Waypoint for two weeks without losing a message or replying twice.

## Next: daily speed

Once the inbox works, the work shifts to how quickly an agent can move through
it.

- Live updates for the queue and the open ticket, without a page reload.
- Canned replies and keyboard shortcuts.
- Tags, saved views, and full-text search.
- Customer pages with their ticket history.
- Ticket merge for duplicate conversations.
- Notification preferences and mentions in internal notes.

## Then: integrations

Waypoint should fit into the tools a team already uses.

- Scoped API keys and a documented JSON API for tickets, messages, and
  customers.
- An OpenAPI document generated from the application routes.
- Signed webhooks with retries and a delivery log.
- Idempotency keys and rate limits for API clients.

## Later: service levels and automation

- Business hours and SLA targets for first response and resolution.
- Overdue indicators in the queue.
- Rules that assign, tag, or reply automatically.
- Round-robin assignment.
- Reports on volume, response time, resolution time, and agent load.
- CSV export.

## Longer term

- PostgreSQL deployments with multiple application instances.
- A customer portal where requesters can view their own tickets.
- Localized public pages and customer email.
- A satisfaction survey on resolution.
- A knowledge base with suggested articles on the ticket form.
- Multiple mailboxes and brand-specific forms.
- Single sign-on through OIDC.
- Integrations such as Slack notifications and issue links.

## Not planned

Waypoint is not a CRM, a live chat product, or a multi-channel contact center.
Billing, a mobile application, and AI features in the product itself are outside
the current direction. The schema is workspace-aware, so a hosted multi-tenant
offering would not require a redesign, but it is not on the roadmap.
