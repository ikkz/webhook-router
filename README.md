# Webhook Router

A universal webhook adapter that normalizes incoming webhook payloads into Markdown and forwards them to multiple downstream targets with platform-specific formatting.

## Background

Many monitoring and alerting tools (like GlitchTip, Sentry, etc.) only support a limited set of webhook targets such as Slack and Discord. However, teams often use different communication platforms (DingTalk, Lark, WeCom, etc.) that aren't on the supported list. This creates a gap when you need to receive alerts in your team's preferred tool.

Webhook Router solves this problem by acting as a universal adapter that:
- **Accepts webhooks from multiple platforms** (ingress compatibility) - supports various webhook formats including Slack, DingTalk, Lark, WeCom, and custom HTTP webhooks
- **Forwards to multiple platforms** (egress compatibility) - converts and delivers messages to any supported target platform
- **Normalizes content to Markdown** - provides a unified intermediate format for easy transformation
- **Manages multiple targets** - send one webhook to many destinations simultaneously

## Features

- Accepts incoming webhooks and normalizes content to Markdown
- Forwards events to multiple targets with platform-specific formatting
- Records delivery results for each target
- Provides a console UI under `/console` and a Basic Auth protected API under `/console/api`
- Supports custom banner/footer for message customization

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
- `--public-ingress-base-url` / `WEBHOOK_ROUTER_PUBLIC_INGRESS_BASE_URL` (optional, e.g. `https://data.example.com/webhooks`)

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

To have the console display the reverse proxy URL, set:
`WEBHOOK_ROUTER_PUBLIC_INGRESS_BASE_URL=https://data.example.com/webhooks`

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
nx run webhook_router:run:debug
```

Run tests:

```bash
nx test webhook_router
```

Run e2e tests:

```bash
nx e2e console-e2e --outputStyle=static
```

Generate OpenAPI + API client:

```bash
nx run api-client:generate
```

## Docs
- `docs/tech-plan.md`: architecture notes
- `docs/adapters/`: adapter formats
