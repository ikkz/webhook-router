# Webhook Router

Webhook Router normalizes incoming webhook payloads into Markdown and forwards them to one or more downstream targets. It includes a lightweight console UI and a basic API for managing endpoints and targets.

## What it does
- Accepts incoming webhooks and normalizes content to Markdown.
- Forwards events to multiple targets and records delivery results.
- Provides a console UI under `/console` and a Basic Auth protected API under `/console/api`.

## Docker
The container expects configuration through environment variables.

Example:

```bash
docker run --rm -p 3000:3000 \
  -e WEBHOOK_ROUTER_USERNAME=your_username \
  -e WEBHOOK_ROUTER_PASSWORD=your_password \
  -v /path/on/host:/app/data \
  ghcr.io/ikkz/webhook-router:latest
```

Docker Compose example: `examples/docker-compose.example.yml`

## Configuration
All CLI flags are also available via environment variables (useful for Docker).

- `--bind` / `WEBHOOK_ROUTER_BIND` (default: `0.0.0.0:3000`)
- `--db-path` / `WEBHOOK_ROUTER_DB_PATH` (default: `data/webhook_router.db`)
- `--username` / `WEBHOOK_ROUTER_USERNAME` (required)
- `--password` / `WEBHOOK_ROUTER_PASSWORD` (required)
- `--swagger-ui` / `WEBHOOK_ROUTER_SWAGGER_UI`
- `--generate-openapi` / `WEBHOOK_ROUTER_GENERATE_OPENAPI`

### Reverse proxy
As long as you set a strong password, exposing the bind port directly to the public internet is safe enough. If you only want to expose specific webhook endpoints publicly, you can use a reverse proxy for path routing. Example Caddy config:

```caddyfile
https://example.com {
        handle_path /webhooks/* {
                rewrite * /ingress{path}
                reverse_proxy localhost:3000
        }

        handle {
                abort
        }
}
```

With this setup, the console shows an ingress URL like:
`http://localhost:3000/ingress/5bc06725-97e9-4cc7-92f9-9258972687cb/lark`

When configuring external platforms, use:
`https://data.example.com/webhooks/5bc06725-97e9-4cc7-92f9-9258972687cb/lark`

## Repo layout
- `apps/webhook_router`: Rust backend (Axum + SQLite)
- `apps/console`: React console UI
- `docs/`: design notes and adapter formats

## HTTP endpoints
- Ingress: `POST /ingress/:endpoint_id/:platform`
- Console UI: `GET /console`
- Console API (Basic Auth): `GET /console/api/...`

## Platform compatibility
| Platform | Ingress message types | Markdown normalization |
| --- | --- | --- |
| DingTalk | text, link, markdown, actionCard (single + buttons), feedCard | Best-effort Markdown from message fields |
| Slack | text, blocks (section/header/divider/image/context/actions/rich_text), attachments, sections | Best-effort Markdown; mrkdwn preserved where possible |
| Lark | text | Uses message content text |
| WeCom | text, markdown, markdown_v2 | Uses content text |
| Custom HTTP | markdown, text (fallback to raw JSON) | Uses provided markdown/text or raw JSON |

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

## Docs
- `docs/tech-plan.md`: architecture notes
- `docs/adapters/`: adapter formats
