use axum::response::{Html, IntoResponse};

/// Embedded console HTML built from apps/console/dist/index.html
/// In release builds, this must exist. In non-release builds, we provide a fallback.
#[cfg(feature = "console")]
const CONSOLE_HTML: &str = include_str!("../../console/dist/index.html");

#[cfg(not(feature = "console"))]
const CONSOLE_HTML: &str = r#"<html><body>Console UI not embedded. Please build with '--features console' to include it.</body></html>"#;

/// Handler to serve the embedded console HTML
pub async fn serve_console() -> impl IntoResponse {
    Html(CONSOLE_HTML)
}
