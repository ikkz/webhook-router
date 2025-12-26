# AGENTS

This repo uses Codex-style coding agents. Keep instructions short, concrete, and scoped to this project.

## Project Layout
- `apps/webhook_router`: Rust backend (Axum + SQLite)
- `apps/console`: React frontend
- `docs/`: design notes and specs

## Backend Conventions
- SQLite only; support `:memory:` for tests.
- No queues; dispatch synchronously.
- Webhook endpoints are `POST /ingress/:endpoint_id/:platform`.
- Use HTTP Basic Auth for `/api`.
- Adapters live in `apps/webhook_router/src/adapters/`.

## Testing
- Prefer unit tests near the code (module `#[cfg(test)]`).
- Run: `cargo test -p webhook_router`.

## Docs
- Update `docs/tech-plan.md` if architecture changes.
- Adapter formats live in `docs/adapters/`.
