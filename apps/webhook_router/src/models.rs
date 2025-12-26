use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct BasicAuth {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Deserialize)]
pub struct CreateTargetRequest {
    pub name: String,
    pub kind: String,
    pub url: String,
    pub headers: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct Target {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub url: String,
    pub headers: Option<Value>,
    pub created_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateEndpointRequest {
    pub name: String,
    pub target_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEndpointRequest {
    pub name: Option<String>,
    pub target_ids: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct Endpoint {
    pub id: String,
    pub name: String,
    pub target_ids: Vec<String>,
    pub created_at: i64,
}

#[derive(Debug, Serialize)]
pub struct EventRecord {
    pub id: String,
    pub endpoint_id: String,
    pub platform: String,
    pub title: Option<String>,
    pub markdown: String,
    pub raw: Value,
    pub created_at: i64,
}

#[derive(Debug, Serialize)]
pub struct DeliveryOutcome {
    pub target_id: String,
    pub status: String,
    pub response_code: Option<u16>,
    pub error: Option<String>,
}
