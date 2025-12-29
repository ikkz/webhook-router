use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use utoipa::OpenApi;
use base64::Engine;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::adapters::{egress_adapter, ingress_adapter};
use crate::db::Db;
use crate::models::{
    BasicAuth, CreateEndpointRequest, CreateTargetRequest, DeliveryOutcome, Endpoint, EventRecord,
    Target, TestSendRequest, UpdateEndpointRequest, UemEvent,
};
use crate::utils::format::format_markdown;

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub auth: BasicAuth,
    pub http: reqwest::Client,
}

#[utoipa::path(
    get,
    path = "/api/auth/check",
    responses(
        (status = 200, description = "Auth check", body = Value),
        (status = 403, description = "Forbidden")
    ),
    security(
        ("basic_auth" = [])
    )
)]
pub async fn check_auth(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "valid": true,
        "username": state.auth.username
    }))
}

pub fn api_router() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/auth/check", get(check_auth))
        .route("/endpoints/:id/targets", post(create_target).get(list_targets))
        .route("/endpoints/:id/targets/:target_id", delete(delete_target))
        .route("/endpoints", post(create_endpoint).get(list_endpoints))
        .route("/endpoints/:id", put(update_endpoint).get(get_endpoint))
        .route("/endpoints/:id/test", post(test_send))
        .route("/events", get(list_events))
}

#[derive(OpenApi)]
#[openapi(
    paths(
        healthz,
        ingress,
        check_auth,
        create_target,
        list_targets,
        delete_target,
        create_endpoint,
        get_endpoint,
        list_endpoints,
        update_endpoint,
        test_send,
        list_events,
    ),
    components(
        schemas(
            UemEvent,
            CreateTargetRequest,
            Target,
            CreateEndpointRequest,
            UpdateEndpointRequest,
            Endpoint,
            EventRecord,
            DeliveryOutcome,
            TestSendRequest,
            AppErrorResponse,
        )
    ),
    tags(
        (name = "webhook-router", description = "Webhook Router API")
    )
)]
pub struct ApiDoc;

#[derive(utoipa::ToSchema)]
#[allow(dead_code)]
pub struct AppErrorResponse {
    pub error: String,
}

#[utoipa::path(
    get,
    path = "/healthz",
    responses(
        (status = 200, description = "Health check", body = String)
    )
)]
pub async fn healthz() -> &'static str {
    "ok"
}

#[utoipa::path(
    post,
    path = "/ingress/{endpoint_id}/{platform}",
    params(
        ("endpoint_id" = String, Path, description = "Endpoint ID"),
        ("platform" = String, Path, description = "Platform name")
    ),
    request_body = Value,
    responses(
        (status = 200, description = "Event processed successfully", body = Value),
        (status = 400, description = "Bad request", body = AppErrorResponse),
        (status = 404, description = "Endpoint not found", body = AppErrorResponse)
    )
)]
pub async fn ingress(
    Path((endpoint_id, platform)): Path<(String, String)>,
    State(state): State<AppState>,
    body: axum::body::Bytes,
) -> Result<Json<Value>, AppError> {
    let payload: Value = serde_json::from_slice(&body).map_err(|err| {
        AppError::bad_request(format!("invalid json payload: {err}"))
    })?;

    if let Some(challenge) = payload.get("challenge").and_then(|value| value.as_str()) {
        if payload.get("type").and_then(|value| value.as_str()) == Some("url_verification") {
            return Ok(Json(json!({ "challenge": challenge })));
        }
    }

    let endpoint = state
        .db
        .get_endpoint(&endpoint_id)
        .await
        .map_err(AppError::from)?;

    let endpoint = endpoint.ok_or_else(|| AppError::not_found("endpoint not found"))?;

    let adapter = ingress_adapter(&platform)
        .ok_or_else(|| AppError::bad_request("unsupported platform"))?;

    let mut event = adapter
        .ingress_to_uem(&payload)
        .map_err(|err| AppError::bad_request(err.message))?;

    if event.id.is_empty() {
        event.id = Uuid::new_v4().to_string();
    }

    // Concatenate banner + markdown + footer with proper formatting
    let mut final_markdown = String::new();
    
    if let Some(banner) = &endpoint.banner {
        if !banner.is_empty() {
            final_markdown.push_str(banner.trim());
            final_markdown.push_str("\n\n");
        }
    }
    
    final_markdown.push_str(event.markdown.trim());
    
    if let Some(footer) = &endpoint.footer {
        if !footer.is_empty() {
            final_markdown.push_str("\n\n");
            final_markdown.push_str(footer.trim());
        }
    }
    
    // Format the concatenated markdown for proper spacing
    event.markdown = format_markdown(&final_markdown)
        .unwrap_or_else(|_| final_markdown);

    state
        .db
        .insert_event(&endpoint.id, &platform, &event)
        .await
        .map_err(AppError::from)?;

    // Fetch targets for this endpoint
    let targets = state.db.list_targets(&endpoint.id).await.map_err(AppError::from)?;
    
    let mut outcomes = Vec::new();
    for target in targets {
        let outcome = dispatch_to_target(&state, &event, &target).await;
        outcomes.push(outcome);
    }

    Ok(Json(json!({
        "event_id": event.id,
        "deliveries": outcomes,
    })))
}

async fn dispatch_to_target(
    state: &AppState,
    event: &UemEvent,
    target: &Target,
) -> DeliveryOutcome {
    let adapter = match egress_adapter(&target.kind) {
        Some(adapter) => adapter,
        None => {
            let _ = state
                .db
                .insert_delivery(&event.id, &target.id, "failed", None, Some("unsupported target".to_string()))
                .await;
            return DeliveryOutcome {
                target_id: target.id.clone(),
                status: "failed".to_string(),
                response_code: None,
                error: Some("unsupported target".to_string()),
            };
        }
    };

    let payload = match adapter.uem_to_egress(event) {
        Ok(payload) => payload,
        Err(err) => {
            let _ = state
                .db
                .insert_delivery(&event.id, &target.id, "failed", None, Some(err.message.clone()))
                .await;
            return DeliveryOutcome {
                target_id: target.id.clone(),
                status: "failed".to_string(),
                response_code: None,
                error: Some(err.message),
            };
        }
    };

    let mut request = state
        .http
        .post(&target.url)
        .header("Content-Type", payload.content_type)
        .json(&payload.body);

    if let Some(headers) = target.headers.as_ref().and_then(|value| value.as_object()) {
        for (key, value) in headers {
            if let Some(value) = value.as_str() {
                request = request.header(key, value);
            }
        }
    }

    let response = request.send().await;
    match response {
        Ok(resp) => {
            let status = if resp.status().is_success() { "sent" } else { "failed" };
            let code = resp.status().as_u16();
            let error = if resp.status().is_success() {
                None
            } else {
                Some(format!("non-success status: {}", resp.status()))
            };
            let _ = state
                .db
                .insert_delivery(&event.id, &target.id, status, Some(code), error.clone())
                .await;
            DeliveryOutcome {
                target_id: target.id.clone(),
                status: status.to_string(),
                response_code: Some(code),
                error,
            }
        }
        Err(err) => {
            let message = err.to_string();
            let _ = state
                .db
                .insert_delivery(&event.id, &target.id, "failed", None, Some(message.clone()))
                .await;
            DeliveryOutcome {
                target_id: target.id.clone(),
                status: "failed".to_string(),
                response_code: None,
                error: Some(message),
            }
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/endpoints/{id}/targets",
    params(
        ("id" = String, Path, description = "Endpoint ID")
    ),
    request_body = CreateTargetRequest,
    responses(
        (status = 200, description = "Target created successfully", body = Target),
        (status = 400, description = "Bad request", body = AppErrorResponse)
    ),
    security(
        ("basic_auth" = [])
    )
)]
async fn create_target(
    Path(endpoint_id): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<CreateTargetRequest>,
) -> Result<Json<Target>, AppError> {
    // Verify endpoint exists
    if state.db.get_endpoint(&endpoint_id).await.map_err(AppError::from)?.is_none() {
        return Err(AppError::not_found("endpoint not found"));
    }

    let target = state.db.create_target(&endpoint_id, req).await.map_err(AppError::from)?;
    Ok(Json(target))
}

#[utoipa::path(
    get,
    path = "/api/endpoints/{id}/targets",
    params(
        ("id" = String, Path, description = "Endpoint ID")
    ),
    responses(
        (status = 200, description = "List of targets", body = [Target])
    ),
    security(
        ("basic_auth" = [])
    )
)]
async fn list_targets(
    Path(endpoint_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Vec<Target>>, AppError> {
    let targets = state.db.list_targets(&endpoint_id).await.map_err(AppError::from)?;
    Ok(Json(targets))
}

#[utoipa::path(
    delete,
    path = "/api/endpoints/{id}/targets/{target_id}",
    params(
         ("id" = String, Path, description = "Endpoint ID"),
         ("target_id" = String, Path, description = "Target ID")
    ),
    responses(
        (status = 204, description = "Target deleted successfully"),
        (status = 404, description = "Target not found", body = AppErrorResponse)
    ),
    security(
        ("basic_auth" = [])
    )
)]
async fn delete_target(
    Path((_endpoint_id, target_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<StatusCode, AppError> {
    let affected = state.db.delete_target(&target_id).await.map_err(AppError::from)?;
    if affected == 0 {
        return Err(AppError::not_found("target not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/endpoints",
    request_body = CreateEndpointRequest,
    responses(
        (status = 200, description = "Endpoint created successfully", body = Endpoint),
        (status = 400, description = "Bad request", body = AppErrorResponse)
    ),
    security(
        ("basic_auth" = [])
    )
)]
async fn create_endpoint(
    State(state): State<AppState>,
    Json(req): Json<CreateEndpointRequest>,
) -> Result<Json<Endpoint>, AppError> {
    let endpoint = state
        .db
        .create_endpoint(req)
        .await
        .map_err(AppError::from)?;
    Ok(Json(endpoint))
}

#[utoipa::path(
    get,
    path = "/api/endpoints",
    responses(
        (status = 200, description = "List of endpoints", body = [Endpoint])
    ),
    security(
        ("basic_auth" = [])
    )
)]
async fn list_endpoints(State(state): State<AppState>) -> Result<Json<Vec<Endpoint>>, AppError> {
    let endpoints = state.db.list_endpoints().await.map_err(AppError::from)?;
    Ok(Json(endpoints))
}

#[utoipa::path(
    put,
    path = "/api/endpoints/{id}",
    params(
        ("id" = String, Path, description = "Endpoint ID")
    ),
    request_body = UpdateEndpointRequest,
    responses(
        (status = 200, description = "Endpoint updated successfully", body = Endpoint),
        (status = 400, description = "Bad request", body = AppErrorResponse),
        (status = 404, description = "Endpoint not found", body = AppErrorResponse)
    ),
    security(
        ("basic_auth" = [])
    )
)]
async fn update_endpoint(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<UpdateEndpointRequest>,
) -> Result<Json<Endpoint>, AppError> {
    if req.name.is_none() && req.banner.is_none() && req.footer.is_none() {
        return Err(AppError::bad_request("no fields to update"));
    }
    let endpoint = state
        .db
        .update_endpoint(&id, req)
        .await
        .map_err(AppError::from)?;
    let endpoint = endpoint.ok_or_else(|| AppError::not_found("endpoint not found"))?;
    Ok(Json(endpoint))
}

#[utoipa::path(
    post,
    path = "/api/endpoints/{id}/test",
    params(
        ("id" = String, Path, description = "Endpoint ID")
    ),
    request_body = TestSendRequest,
    responses(
        (status = 200, description = "Test message sent successfully", body = Value),
        (status = 404, description = "Endpoint not found", body = AppErrorResponse)
    ),
    security(
        ("basic_auth" = [])
    )
)]
async fn test_send(
    Path(endpoint_id): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<TestSendRequest>,
) -> Result<Json<Value>, AppError> {
    let endpoint = state
        .db
        .get_endpoint(&endpoint_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::not_found("endpoint not found"))?;

    let mut event = UemEvent {
        id: Uuid::new_v4().to_string(),
        source: "test".to_string(),
        timestamp: now_timestamp(),
        title: Some("Test Message".to_string()),
        markdown: req.markdown,
        raw: json!({"test": true}),
        meta: json!({}),
    };

    // Apply banner/footer concatenation with proper formatting
    let mut final_markdown = String::new();
    
    if let Some(banner) = &endpoint.banner {
        if !banner.is_empty() {
            final_markdown.push_str(banner.trim());
            final_markdown.push_str("\n\n");
        }
    }
    
    final_markdown.push_str(event.markdown.trim());
    
    if let Some(footer) = &endpoint.footer {
        if !footer.is_empty() {
            final_markdown.push_str("\n\n");
            final_markdown.push_str(footer.trim());
        }
    }
    
    // Format the concatenated markdown for proper spacing
    event.markdown = format_markdown(&final_markdown)
        .unwrap_or_else(|_| final_markdown);

    // Fetch targets and dispatch
    let targets = state.db.list_targets(&endpoint_id).await.map_err(AppError::from)?;
    let mut outcomes = Vec::new();
    for target in targets {
        let outcome = dispatch_to_target(&state, &event, &target).await;
        outcomes.push(outcome);
    }

    Ok(Json(json!({
        "event_id": event.id,
        "deliveries": outcomes,
    })))
}

fn now_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

#[utoipa::path(
    get,
    path = "/api/endpoints/{id}",
    params(
        ("id" = String, Path, description = "Endpoint ID")
    ),
    responses(
        (status = 200, description = "Endpoint details", body = Endpoint),
        (status = 404, description = "Endpoint not found", body = AppErrorResponse)
    ),
    security(
        ("basic_auth" = [])
    )
)]
async fn get_endpoint(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Endpoint>, AppError> {
    let endpoint = state
        .db
        .get_endpoint(&id)
        .await
        .map_err(AppError::from)?;
    let endpoint = endpoint.ok_or_else(|| AppError::not_found("endpoint not found"))?;
    Ok(Json(endpoint))
}

#[utoipa::path(
    get,
    path = "/api/events",
    responses(
        (status = 200, description = "List of events", body = [EventRecord])
    ),
    security(
        ("basic_auth" = [])
    )
)]
async fn list_events(State(state): State<AppState>) -> Result<Json<Vec<EventRecord>>, AppError> {
    let events = state.db.list_events().await.map_err(AppError::from)?;
    Ok(Json(events))
}

pub async fn basic_auth(
    State(state): State<AppState>,
    request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Result<Response, StatusCode> {
    if is_authorized(&state.auth, request.headers()) {
        return Ok(next.run(request).await);
    }

    // Return 403 Forbidden without WWW-Authenticate header to prevent browser's native auth dialog
    // Frontend will handle redirecting to login page
    Ok(StatusCode::FORBIDDEN.into_response())
}

fn is_authorized(auth: &BasicAuth, headers: &HeaderMap) -> bool {
    let header = match headers.get(axum::http::header::AUTHORIZATION) {
        Some(value) => value,
        None => return false,
    };
    let header = match header.to_str() {
        Ok(value) => value,
        Err(_) => return false,
    };
    let encoded = header.strip_prefix("Basic ").unwrap_or("");
    if encoded.is_empty() {
        return false;
    }
    let decoded = match base64::engine::general_purpose::STANDARD.decode(encoded) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };
    let decoded = match String::from_utf8(decoded) {
        Ok(value) => value,
        Err(_) => return false,
    };
    let mut parts = decoded.splitn(2, ':');
    let username = parts.next().unwrap_or("");
    let password = parts.next().unwrap_or("");
    username == auth.username && password == auth.password
}

#[derive(Debug)]
pub struct AppError {
    status: StatusCode,
    message: String,
}

impl AppError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.into(),
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: err.to_string(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = Json(json!({ "error": self.message }));
        (self.status, body).into_response()
    }
}
