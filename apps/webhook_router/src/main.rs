use axum::middleware;
use axum::routing::{get, post};
use axum::Router;
use clap::Parser;
use tracing_subscriber::EnvFilter;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod adapters;
mod console_handlers;
mod db;
mod handlers;
mod models;
mod utils;

use handlers::{api_router, basic_auth, healthz, ingress, AppState, ApiDoc};
use models::BasicAuth;

#[derive(Debug, Parser)]
#[command(name = "webhook-router", version)]
struct Args {
    #[arg(long, default_value = "0.0.0.0:3000")]
    bind: String,
    #[arg(long, default_value = "webhook_router.db")]
    db_path: String,
    #[arg(long)]
    username: String,
    #[arg(long)]
    password: String,
    #[arg(long)]
    generate_openapi: bool,
    #[arg(long)]
    swagger_ui: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,tower_http=debug")),
        )
        .init();

    let args = Args::parse();

    if args.generate_openapi {
        let doc = ApiDoc::openapi();
        println!("{}", doc.to_pretty_json().unwrap());
        return Ok(());
    }

    let db = db::Db::connect(&args.db_path).await?;

    let state = AppState {
        db,
        auth: BasicAuth {
            username: args.username,
            password: args.password,
        },
        http: reqwest::Client::new(),
    };

    // Protected API routes with authentication
    let protected_api = api_router()
        .layer(middleware::from_fn_with_state(state.clone(), basic_auth));

    // Console router: HTML is public, API is protected
    let console = Router::<AppState>::new()
        .route("/", get(console_handlers::serve_console))
        .nest("/api", protected_api);

    let mut app = Router::<AppState>::new()
        // Public routes (no authentication)
        .route("/healthz", get(healthz))
        .route("/ingress/:endpoint_id/:platform", post(ingress));

    if args.swagger_ui {
        app = app.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));
    }

    let app = app
        // Console routes (HTML public, API protected)
        .nest("/console", console)
        .with_state(state)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(&args.bind).await?;
    tracing::info!("listening on {}", args.bind);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("received Ctrl+C signal, shutting down gracefully");
        },
        _ = terminate => {
            tracing::info!("received SIGTERM signal, shutting down gracefully");
        },
    }
}
