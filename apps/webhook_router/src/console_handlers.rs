use axum::response::{Html, IntoResponse};
use axum::extract::State;

use crate::handlers::AppState;

/// Embedded console HTML built from apps/console/dist/index.html
/// In release builds, this must exist. In non-release builds, we provide a fallback.
#[cfg(feature = "console")]
const CONSOLE_HTML: &str = include_str!("../../console/dist/index.html");

#[cfg(not(feature = "console"))]
const CONSOLE_HTML: &str = r#"<html><body>Console UI not embedded. Please build with '--features console' to include it.</body></html>"#;

pub fn build_console_html(public_ingress_base_url: Option<&str>) -> String {
    let mut html = CONSOLE_HTML.to_string();

    if let Some(ingress_base_url) = public_ingress_base_url {
        let trimmed = ingress_base_url.trim().trim_end_matches('/');
        if !trimmed.is_empty() {
            let escaped = serde_json::to_string(trimmed).unwrap_or_else(|_| "\"\"".to_string());
            let script = format!(
                "<script>window.__WEBHOOK_ROUTER_INGRESS_BASE_URL__={};</script>",
                escaped
            );
            if let Some(idx) = html.find("</head>") {
                html.insert_str(idx, &script);
            } else {
                html.push_str(&script);
            }
        }
    }

    html
}

/// Handler to serve the embedded console HTML
pub async fn serve_console(State(state): axum::extract::State<AppState>) -> impl IntoResponse {
    Html(state.console_html.as_ref().to_string())
}
