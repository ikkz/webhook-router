# AGENTS

This repo uses Codex-style coding agents. Keep instructions short, concrete, and scoped to this project.

## Nx Commands

- All `nx` commands MUST be prefixed with `pnpm exec`.
- **IMPORTANT**: Always add `--outputStyle=static` to any `nx` command to ensure logs are visible in the terminal.

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
- Run backend tests: `pnpm exec nx test webhook_router`.
- Run e2e tests: `pnpm exec nx e2e console-e2e`.

## Docs

- Update `docs/tech-plan.md` if architecture changes.
- Adapter formats live in `docs/adapters/`.

## Client Generation

- To generate the API client: `pnpm exec nx run api-client:generate`




<!-- nx configuration start-->
<!-- Leave the start & end comments to automatically receive updates. -->

# General Guidelines for working with Nx

- When running tasks (for example build, lint, test, e2e, etc.), always prefer running the task through `nx` (i.e. `nx run`, `nx run-many`, `nx affected`) instead of using the underlying tooling directly
- You have access to the Nx MCP server and its tools, use them to help the user
- When answering questions about the repository, use the `nx_workspace` tool first to gain an understanding of the workspace architecture where applicable.
- When working in individual projects, use the `nx_project_details` mcp tool to analyze and understand the specific project structure and dependencies
- For questions around nx configuration, best practices or if you're unsure, use the `nx_docs` tool to get relevant, up-to-date docs. Always use this instead of assuming things about nx configuration
- If the user needs help with an Nx configuration or project graph error, use the `nx_workspace` tool to get any errors

<!-- nx configuration end-->
