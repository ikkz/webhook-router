FROM rust:1.86-bookworm AS build

RUN apt-get update \
  && apt-get install -y --no-install-recommends \
    ca-certificates \
    nodejs \
    npm \
    pkg-config \
    libsqlite3-dev \
  && rm -rf /var/lib/apt/lists/*

RUN npm install -g pnpm@10

WORKDIR /repo
COPY . .

RUN pnpm install --frozen-lockfile
RUN pnpm exec nx build webhook_router --configuration=production

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
  && apt-get install -y --no-install-recommends \
    ca-certificates \
    libsqlite3-0 \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=build /repo/dist/target/webhook_router/release/webhook_router /app/webhook_router

EXPOSE 3000
ENTRYPOINT ["/app/webhook_router"]
