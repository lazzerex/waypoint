# Product

## Vision

A shared inbox and helpdesk that a small team can set up in an afternoon and still enjoy using a year later. Fast pages, clear ownership of every conversation, and an API so it can fit into the rest of the stack.

## Target users

| Persona | Description | Needs |
|---|---|---|
| **Agent** (Priya) | Support or ops person handling 20-80 conversations a day | Fast list, keyboard-friendly, canned replies, clear "what's mine" view |
| **Admin** (Marcus) | Founder or team lead who also sets things up | Invite staff, connect the support address, see volume and response times |
| **Customer** (Dana) | Someone who needs help | Easy way to ask, replies that land in the same email thread, no account required |
| **Integrator** (Sam) | Developer wiring the helpdesk into other systems | Stable API, OpenAPI spec, signed webhooks |

## Principles

1. **Email is the primary channel.** Customers never need an account.
2. **One owner per ticket.** Unassigned is a visible, first-class state.
3. **Fast by default.** Server-rendered pages, minimal client JS.
4. **Boring and reliable.** Never lose an inbound email. Never send a duplicate reply.
5. **API parity.** Anything staff can do in the UI is possible through the API.

## MVP scope

### In scope

- Ticket creation via web form and inbound email
- Email threading (customer replies attach to existing tickets)
- Staff login, roles (`admin`, `agent`)
- Ticket list with filters: status, assignee, priority
- Ticket detail: conversation thread, internal notes, activity log
- Actions: assign, change status, change priority, reply, add note
- Outbound email replies from the support address
- Attachments (receive, store, download) with size limits

### Out of scope for MVP

- Multiple workspaces / multi-tenant signup
- Billing
- Knowledge base and customer portal
- Chat, social, or phone channels
- AI features
- Mobile app

## User stories (MVP)

### Customer

- As a customer, I can submit a ticket with my email, name, subject, and message so that support can help me.
- As a customer, I receive a confirmation email containing the ticket number.
- As a customer, when I reply to any email from support, my reply is added to the same ticket.

### Agent

- As an agent, I can see all open tickets and filter by status, assignee, and priority.
- As an agent, I can see "My tickets" and "Unassigned" at a glance.
- As an agent, I can assign a ticket to myself or a teammate.
- As an agent, I can reply to the customer and have it sent from the support address.
- As an agent, I can leave an internal note that the customer never sees.
- As an agent, I can resolve a ticket, and it reopens automatically if the customer replies.
- As an agent, I can see the full history of a ticket: who changed what and when.

### Admin

- As an admin, I can invite and deactivate agents.
- As an admin, I can configure the support mailbox (address, display name, signature).
- As an admin, I can create API keys and webhooks (Phase 3).

## Acceptance criteria highlights

- An inbound email is stored exactly once even if the provider retries the webhook (idempotent on `Message-ID`).
- A reply sent by an agent threads correctly in Gmail, Outlook, and Apple Mail.
- Auto-replies and bounces (out-of-office, delivery failures) do not create new tickets or loops.
- Ticket list loads in under 200 ms for 10,000 tickets in a workspace.

## Success metrics

- Median first response time
- Median time to resolution
- Percentage of tickets unassigned for more than N minutes
- Inbound email processing failures (target: zero lost messages)

## Open questions

- Which inbound email provider to support first? (Postmark is the simplest webhook payload.)
- Per-agent signatures or per-mailbox only?
- Should customers be able to close tickets themselves via a link?
