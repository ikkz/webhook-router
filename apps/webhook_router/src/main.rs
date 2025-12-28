use axum::middleware;
use axum::routing::{get, post};
use axum::Router;
use clap::Parser;
use tracing_subscriber::EnvFilter;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod adapters;
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
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

    let api = api_router().layer(middleware::from_fn_with_state(state.clone(), basic_auth));

    let app = Router::<AppState>::new()
        .route("/healthz", get(healthz))
        .route("/ingress/:endpoint_id/:platform", post(ingress))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/api", api)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&args.bind).await?;
    tracing::info!("listening on {}", args.bind);
    axum::serve(listener, app).await?;
    Ok(())
}
