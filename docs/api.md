# API and webhooks

Waypoint exposes a JSON API for integrations and signed webhooks for event
delivery. The document below describes the intended public contract. The
OpenAPI document served by the application is authoritative once those routes
are implemented.

## Shape

- Base path: `/api/v1`
- Content type: `application/json`
- Authentication: `Authorization: Bearer <api_key>`
- Versioning: path-based; breaking changes move to `/api/v2`
- Timestamps: ISO 8601 in UTC, for example `2026-11-24T09:30:00Z`
- Ticket identifiers: the workspace ticket number; other resources use UUIDs

## Authentication

An administrator creates API keys in the staff settings. The full key is shown
once, at creation. The server stores only a hash and a short display prefix.

```http
GET /api/v1/tickets HTTP/1.1
Authorization: Bearer wpt_live_xxxxxxxxxxxxxxxxxxxxxxxx
```

| Scope | Access |
|---|---|
| `tickets:read` | Read tickets, messages, and customers |
| `tickets:write` | Create tickets, reply, change status, assign |
| `customers:write` | Create and update customers |
| `webhooks:manage` | Manage webhook endpoints |

Missing or invalid credentials return `401`. A valid key without the required
scope returns `403`.

API requests do not use cookies. The `/api` prefix therefore disables the
browser-origin policy and relies on bearer authentication instead.

## Requests and responses

### Pagination

Lists use opaque cursors rather than offsets.

```http
GET /api/v1/tickets?limit=50&cursor=eyJ...
```

```json
{
  "data": [],
  "next_cursor": "eyJ...",
  "has_more": true
}
```

The default limit is 25 and the maximum is 100.

### Errors

```json
{
  "error": {
    "code": "validation_failed",
    "message": "Subject is required.",
    "details": { "field": "subject" }
  }
}
```

| Status | Code | Meaning |
|---|---|---|
| 400 | `bad_request` | Malformed JSON or parameters |
| 401 | `unauthorized` | Missing or invalid key |
| 403 | `forbidden` | Key lacks the required scope |
| 404 | `not_found` | Resource absent from this workspace |
| 409 | `conflict` | Invalid state transition or duplicate resource |
| 422 | `validation_failed` | Field-level validation failure |
| 429 | `rate_limited` | Retry later; `Retry-After` is set |
| 500 | `internal` | Unexpected server error |

### Idempotency

Creating `POST` requests accept an `Idempotency-Key` header. Repeating the same
key within 24 hours returns the original response rather than creating a second
resource.

### Rate limits

The default allowance is 600 requests per minute per key. Responses carry
`X-RateLimit-Limit`, `X-RateLimit-Remaining`, and `X-RateLimit-Reset`.

## Resources

### Ticket

```json
{
  "number": 1042,
  "subject": "Can't log in",
  "status": "open",
  "priority": "normal",
  "channel": "email",
  "requester": { "id": "7c1e...", "email": "dana@example.com", "name": "Dana" },
  "assignee": { "id": "a93b...", "name": "Priya" },
  "tags": ["billing"],
  "first_response_at": "2026-11-24T09:41:00Z",
  "resolved_at": null,
  "created_at": "2026-11-24T09:30:00Z",
  "updated_at": "2026-11-24T09:41:00Z"
}
```

### Message

```json
{
  "id": "5d2f...",
  "ticket_number": 1042,
  "kind": "agent",
  "author": { "type": "user", "id": "a93b...", "name": "Priya" },
  "body_text": "Hi Dana, ...",
  "attachments": [
    {
      "id": "e1c0...",
      "filename": "screenshot.png",
      "content_type": "image/png",
      "size_bytes": 48211
    }
  ],
  "created_at": "2026-11-24T09:41:00Z"
}
```

A message kind is `customer`, `agent`, `note`, or `system`.

## Endpoints

### Tickets

| Method | Path | Scope | Purpose |
|---|---|---|---|
| `GET` | `/tickets` | `tickets:read` | List and filter tickets |
| `POST` | `/tickets` | `tickets:write` | Create a ticket |
| `GET` | `/tickets/{number}` | `tickets:read` | Read one ticket |
| `PATCH` | `/tickets/{number}` | `tickets:write` | Change status, priority, assignee, or subject |
| `GET` | `/tickets/{number}/messages` | `tickets:read` | List the thread |
| `POST` | `/tickets/{number}/messages` | `tickets:write` | Add a reply or internal note |

List filters include `status`, `priority`, `assignee_id` (`none` for
unassigned), `requester_email`, `tag`, `q`, `created_after`, and
`created_before`.

Create a ticket:

```json
{
  "requester": { "email": "dana@example.com", "name": "Dana" },
  "subject": "Can't log in",
  "message": "I keep getting an error when I sign in.",
  "priority": "normal",
  "mailbox": "help@example.com"
}
```

Add a reply:

```json
{
  "kind": "agent",
  "body_text": "Thanks Dana, try resetting your password.",
  "send_email": true
}
```

A `note` kind is internal and never sends mail. Invalid status transitions return
`409`.

### Customers

| Method | Path | Scope | Purpose |
|---|---|---|---|
| `GET` | `/customers` | `tickets:read` | List customers, optionally by email |
| `POST` | `/customers` | `customers:write` | Create a customer |
| `GET` | `/customers/{id}` | `tickets:read` | Read one customer |
| `PATCH` | `/customers/{id}` | `customers:write` | Update name or notes |

### Webhook endpoints

| Method | Path | Scope | Purpose |
|---|---|---|---|
| `GET` | `/webhooks` | `webhooks:manage` | List endpoints |
| `POST` | `/webhooks` | `webhooks:manage` | Register an endpoint |
| `DELETE` | `/webhooks/{id}` | `webhooks:manage` | Remove an endpoint |
| `GET` | `/webhooks/{id}/deliveries` | `webhooks:manage` | List recent attempts |

## Webhook events

| Event | Fires when |
|---|---|
| `ticket.created` | A ticket is created through any channel |
| `ticket.updated` | Subject or priority changes |
| `ticket.assigned` | The assignee changes |
| `ticket.status_changed` | The status changes |
| `message.created` | A customer or agent message is added |

Internal notes do not emit `message.created` by default.

### Envelope

```json
{
  "id": "evt_01JD...",
  "type": "ticket.status_changed",
  "created_at": "2026-11-24T10:02:11Z",
  "workspace": "acme",
  "data": {
    "ticket": { "number": 1042, "status": "resolved" },
    "previous": { "status": "open" }
  }
}
```

### Signature

Each delivery carries:

```http
X-Waypoint-Event: ticket.status_changed
X-Waypoint-Delivery: dlv_01JD...
X-Waypoint-Signature: t=1763978531,v1=5257a869e7ecebeda32affa62cdca3fa51cad7e77a0e56ff536d0ce8e108d8bd
```

The signature is `HMAC-SHA256(secret, "{t}.{raw_body}")`. Receivers compare it in
constant time and reject timestamps older than five minutes.

### Delivery

A `2xx` response within ten seconds counts as success. Failed deliveries retry
after 1 minute, 5 minutes, 30 minutes, 2 hours, and 12 hours. Repeated failures
disable the endpoint and notify administrators. Delivery is at least once, so
receivers should deduplicate on the event `id`.

## OpenAPI

The application enables Topcoat's `openapi` feature and derives schemas with
`schemars`. Shared security and error settings live on the `/api` layer; each
route adds its operation metadata.

The document is available at `GET /openapi.json`. CI regenerates it and compares
it with the committed copy. This turns an accidental breaking change into a
reviewable diff.

Internal routes, including the inbound mail webhook and health checks, are
hidden from the public document.
