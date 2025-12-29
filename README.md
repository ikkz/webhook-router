# Webhook Router

Webhook Router normalizes incoming webhook payloads into Markdown and forwards them to one or more downstream targets. It includes a lightweight console UI and a basic API for managing endpoints and targets.

## What it does
- Accepts incoming webhooks and normalizes content to Markdown.
- Forwards events to multiple targets and records delivery results.
- Provides a console UI under `/console` and a Basic Auth protected API under `/console/api`.

## Repo layout
- `apps/webhook_router`: Rust backend (Axum + SQLite)
- `apps/console`: React console UI
- `docs/`: design notes and adapter formats

## HTTP endpoints
- Ingress: `POST /ingress/:endpoint_id/:platform`
- Console UI: `GET /console`
- Console API (Basic Auth): `GET /console/api/...`

## Configuration
All CLI flags are also available via environment variables (useful for Docker).

- `--bind` / `WEBHOOK_ROUTER_BIND` (default: `0.0.0.0:3000`)
- `--db-path` / `WEBHOOK_ROUTER_DB_PATH` (default: `webhook_router.db`)
- `--username` / `WEBHOOK_ROUTER_USERNAME` (required)
- `--password` / `WEBHOOK_ROUTER_PASSWORD` (required)
- `--swagger-ui` / `WEBHOOK_ROUTER_SWAGGER_UI`
- `--generate-openapi` / `WEBHOOK_ROUTER_GENERATE_OPENAPI`

## Local development
Install dependencies:

```bash
pnpm install
```

Run the backend (builds the console as a dependency):

```bash
pnpm exec nx run webhook_router:run:debug
```

Run tests:

```bash
pnpm exec nx run webhook_router:test
```

Generate OpenAPI + API client:

```bash
pnpm exec nx run @webhook-router/api-client:generate
```

## Docker
The container expects configuration through environment variables.

Example:

```bash
docker run --rm -p 3000:3000 \
  -e WEBHOOK_ROUTER_USERNAME=admin \
  -e WEBHOOK_ROUTER_PASSWORD=admin \
  -e WEBHOOK_ROUTER_DB_PATH=/data/webhook_router.db \
  -e WEBHOOK_ROUTER_BIND=0.0.0.0:3000 \
  webhook-router:latest
```

## Docs
- `docs/tech-plan.md`: architecture notes
- `docs/adapters/`: adapter formats
