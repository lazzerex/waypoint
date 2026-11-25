# Overview

Waypoint is a shared inbox and helpdesk for small teams. It gives a support
address a real queue: every conversation becomes a ticket with an owner, a
status, and a complete history, while customers keep using plain email.

## The problem

A shared mailbox works until more than one person answers it. At that point the
team loses track of who replied, what is still open, and which messages belong
together. Dedicated helpdesk products solve this, but they usually bring a large
service footprint, per-seat pricing, and configuration that a small team does
not need.

Waypoint occupies the space between those options. It is a self-hosted
application with the workflows a small support team actually uses, packaged as
one Rust binary and a database.

## The approach

- **Email stays the customer's interface.** Customers write to a support
  address and reply to the notification they receive. No account is required.
- **Every ticket has a visible owner.** Unassigned is a real state, not an
  accidental one.
- **The interface is server-rendered.** Pages load quickly, work without a large
  client bundle, and remain usable on modest hardware.
- **Side effects are durable.** Outbound mail, inbound processing, and webhook
  delivery run through a persistent queue rather than inside a page render.
- **The API mirrors the interface.** Anything staff can do in the UI has a JSON
  equivalent, documented from the routes that serve it.

## Who it serves

| Role | Needs |
|---|---|
| Support agent | A fast queue, clear ownership, canned replies, and a reliable history |
| Team lead or founder | Staff accounts, mailbox settings, and a view of response times |
| Customer | A simple way to ask for help and replies that arrive in the same thread |
| Integrator | A stable API, an OpenAPI document, and signed webhook deliveries |

## Scope

**Inbox.** A public web form and inbound email both create tickets. Replies are
matched to the existing conversation using a signed reply address and standard
mail headers.

**Triage.** The queue can be filtered by status, assignee, and priority. Staff
can assign tickets, change state, add internal notes, and see who changed what.

**Conversation.** Each ticket holds customer messages, agent replies, and
internal notes. Outbound replies are sent from the support address and preserve
the email thread.

**Team.** Staff accounts have an `admin` or `agent` role. Administrators manage
users and mailbox settings.

**Integrations.** A JSON API covers tickets, messages, and customers. Webhooks
deliver ticket events with an HMAC signature and retry schedule.

## Outside the current scope

Waypoint is not a general CRM, a live chat product, or a multi-channel contact
center. There is no billing, no mobile application, and no AI feature in the
product itself. Multi-workspace hosted signup is also out of scope; the schema
is workspace-aware so it does not have to be retrofitted later.

## Signals of success

- An inbound message is stored once, even when the provider retries delivery.
- A reply from an agent threads correctly in Gmail, Outlook, and Apple Mail.
- Automated mail and bounces never create tickets or mail loops.
- A workspace with ten thousand tickets loads its queue in under 200 ms.
