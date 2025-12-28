use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Debug, Clone)]
pub struct BasicAuth {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, ToSchema)]
pub struct UemEvent {
    pub id: String,
    pub source: String,
    pub timestamp: i64,
    pub title: Option<String>,
    pub markdown: String,
    pub raw: Value,
    pub meta: Value,
}

#[derive(Debug, Clone)]
pub struct OutgoingPayload {
    pub body: Value,
    pub content_type: &'static str,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTargetRequest {
    pub name: String,
    pub kind: String,
    pub url: String,
    pub headers: Option<Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Target {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub url: String,
    pub headers: Option<Value>,
    pub created_at: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateEndpointRequest {
    pub name: String,
    pub target_ids: Vec<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateEndpointRequest {
    pub name: Option<String>,
    pub target_ids: Option<Vec<String>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Endpoint {
    pub id: String,
    pub name: String,
    pub target_ids: Vec<String>,
    pub created_at: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EventRecord {
    pub id: String,
    pub endpoint_id: String,
    pub platform: String,
    pub title: Option<String>,
    pub markdown: String,
    pub raw: Value,
    pub created_at: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DeliveryOutcome {
    pub target_id: String,
    pub status: String,
    pub response_code: Option<u16>,
    pub error: Option<String>,
}
