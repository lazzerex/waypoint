# Contributing

It explains how the project is organized and what to check before opening a pull request.

## Ground rules

1. **Keep domain logic framework-free.** Nothing in `src/domain` imports `topcoat` or the ORM. See [architecture.md](docs/architecture.md).
2. **Every query is workspace-scoped.** Repository functions take a `WorkspaceId`. No exceptions.
3. **Authorize inside every shard and procedure.** Topcoat's shard and procedure endpoints bypass page and layout guards, so a guard on the surrounding page does not protect them. Services take an `Actor` and check permissions themselves.
4. **Use `href!` for every internal link, form action, and redirect.** Never hand-write URLs.
5. **Write a decision record** in [decisions.md](docs/decisions.md) for any choice that would be expensive to reverse.
6. **Update docs in the same PR** as the behavior change (data model, API, roadmap checkboxes).

## Development workflow

```bash
topcoat dev                                     # run locally with reload
cargo test                                      # all tests
cargo clippy --all-targets -- -D warnings       # lints must pass
cargo fmt --all
topcoat fmt --rustfmt --check                   # macro bodies (view!, etc.) must be formatted
```

CI runs the same four checks. Run `topcoat fmt --rustfmt` locally to fix formatting.

## Topcoat conventions to follow

- Handlers, pages, layouts, and components take `cx: &Cx` as the first parameter, even if unused.
- Prefer module-based routing (`module_router!()`) so the module tree is the route table. Use `_group` modules (e.g. `_staff`, `_public`) to scope layouts and layers without adding URL segments.
- Use `#[memoize]` helpers (e.g. `current_actor(cx)`) for per-request loads instead of passing data through layers.
- Keep fixed text and attributes as literals inside `view!` so escaping happens at compile time.
- Use `see_other(href!(...).resolve(cx))` (POST/Redirect/GET) after successful form posts. Use POST for all mutations.
- Never capture secrets in `$(...)` expressions or shard arguments. Captures are visible to the browser.
- Validate everything that comes from the browser, including values restored into signals and shard or procedure inputs.

## Branches and commits

- `main` is always releasable.
- Short-lived branches: `feat/<topic>`, `fix/<topic>`, `docs/<topic>`.
- [Conventional Commits](https://www.conventionalcommits.org/): `feat(tickets): add assignee filter`.
- Prefer small PRs that do one thing.

## Testing strategy

| Layer | Approach |
|---|---|
| `domain` | Pure unit tests: status transitions, SLA math, reply-token signing |
| `services` | Integration tests against a real test database, one transaction per test |
| `infra/mail` | Fixture-driven tests using real `.eml` samples (threading, quoting, attachments, charsets) |
| Outbound email | Topcoat's in-memory mail transport; assert on recipients, headers, and body |
| `api` | Request/response tests per endpoint, including auth and error shape |
| `app` (UI) | Smoke tests for critical flows (submit ticket, reply, assign). Start from `Cx::new()` for component-level tests |

Put sample emails in `tests/fixtures/emails/`. Every threading bug gets a new fixture first.

## Adding a feature (checklist)

- [ ] Update [data-model.md](docs/data-model.md); add a migration if the schema changes
- [ ] Domain types and rules in `src/domain`
- [ ] Service function in `src/services` with tests
- [ ] Page, route, or component under `src/app` (and `topcoat ui add` for any new base component, committing `components.toml`)
- [ ] API endpoint with an `Operation` for OpenAPI, if exposed
- [ ] Write a `ticket_events` row and enqueue a webhook job if it changes ticket state
- [ ] Update [api.md](docs/api.md) and [roadmap.md](docs/roadmap.md)

## Migrations

- Change the Toasty models, then `cargo run -- toasty migration generate`, review the output, and commit it.
- Apply with `cargo run -- toasty migration apply`.
- Never edit a migration that has shipped.
- Prefer backwards-compatible steps: add columns nullable, backfill, then tighten.
- Keep SQL portable between SQLite and PostgreSQL unless a repository is explicitly dialect-specific.

## Dependency policy

Topcoat is pre-1.0. Pin it (and Toasty) to an exact version in `Cargo.toml`, read release notes before bumping, and upgrade in a dedicated PR. Run the OpenAPI diff check and the full test suite on every bump. When Topcoat UI components are updated upstream, use `topcoat ui list` to see what changed and merge deliberately: `--overwrite` replaces local edits.

## Reporting security issues

Do not open a public issue. Email the maintainer (address TBD) with details.
