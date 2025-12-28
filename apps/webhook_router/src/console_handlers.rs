use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::Router;

use crate::handlers::{api_router, AppState};

/// Embedded console HTML built from apps/console/dist/index.html
const CONSOLE_HTML: &str = include_str!("../../console/dist/index.html");

/// Handler to serve the embedded console HTML
pub async fn serve_console() -> impl IntoResponse {
    Html(CONSOLE_HTML)
}

/// Console router that includes:
/// - GET / -> serves embedded HTML (will be mounted at /console) - NO AUTH
/// - /api/* -> all API routes (will be mounted at /console/api/*) - WITH AUTH (applied in main.rs)
/// 
/// Note: The /api subrouter will have authentication middleware applied in main.rs
pub fn console_router() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/", get(serve_console))
        .nest("/api", api_router())
}
