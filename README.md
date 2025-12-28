# Webhook Router (WIP)

Webhook Router helps you unify webhook messages from different platforms, convert them into a standard Markdown format, and forward them to one or more targets. It aims to make “many webhook formats in, one normalized message out” simple and repeatable.

This project is currently **work in progress**. Core routing is being built; the console and full platform coverage are still evolving.

## What This Project Does
- Accepts incoming webhooks from popular platforms.
- Normalizes content to Markdown with minimal metadata.
- Forwards to one or more downstream webhook targets.

## Markdown Compatibility
The system ensures your messages look great on any platform by automatically converting standard Markdown:
- **Slack**: Converts standard Markdown (e.g., `**bold**`, `[link](url)`) to Slack's native `mrkdwn` syntax.
- **Lark / Feishu**: Transforms Markdown into Lark's JSON "Post" (Rich Text) format to support native styling.
- **WeCom / DingTalk**: Optimizes formatting for their respective Markdown subsets.

## Status
- Backend routing: in active development.
- Console UI: in progress.
- Specs and formats: tracked under `docs/`.

## Repo Layout
- `apps/webhook_router`: backend service
- `apps/console`: management console
- `docs/`: specs and adapter formats

## Docs
- `docs/tech-plan.md`: architecture notes
- `docs/adapters/`: platform webhook formats
