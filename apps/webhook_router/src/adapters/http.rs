use serde_json::{json, Value};
use uuid::Uuid;

use crate::adapters::{AdapterError, WebhookAdapter};
use crate::models::{OutgoingPayload, UemEvent};

#[derive(Debug)]
pub struct HttpAdapter;

impl WebhookAdapter for HttpAdapter {
    fn ingress_to_uem(&self, payload: &Value) -> Result<UemEvent, AdapterError> {
        let markdown = payload
            .get("markdown")
            .and_then(|value| value.as_str())
            .or_else(|| payload.get("text").and_then(|value| value.as_str()))
            .map(String::from)
            .unwrap_or_else(|| payload.to_string());

        let event_id = payload
            .get("id")
            .and_then(|value| value.as_str())
            .map(String::from)
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let timestamp = payload
            .get("timestamp")
            .and_then(value_to_i64)
            .unwrap_or_else(now_timestamp);

        Ok(UemEvent {
            id: event_id,
            source: "custom".to_string(),
            timestamp,
            title: payload
                .get("title")
                .and_then(|value| value.as_str())
                .map(String::from),
            markdown,
            raw: payload.clone(),
            meta: json!({}),
        })
    }

    fn uem_to_egress(&self, event: &UemEvent) -> Result<OutgoingPayload, AdapterError> {
        Ok(OutgoingPayload {
            body: json!({
                "id": event.id,
                "source": event.source,
                "timestamp": event.timestamp,
                "title": event.title,
                "markdown": event.markdown,
                "meta": event.meta,
                "raw": event.raw,
            }),
            content_type: "application/json",
        })
    }
}

fn now_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

fn value_to_i64(value: &Value) -> Option<i64> {
    value
        .as_i64()
        .or_else(|| value.as_str().and_then(|raw| raw.parse::<i64>().ok()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;
    use serde_json::json;

    #[test]
    fn http_ingress_to_uem() {
        let adapter = HttpAdapter;
        let payload = json!({ "id": "evt-1", "markdown": "hello", "timestamp": 123 });
        let event = adapter.ingress_to_uem(&payload).expect("uem");
        assert_yaml_snapshot!(
            "adapters_http_ingress",
            json!({
                "id": event.id,
                "source": event.source,
                "timestamp": event.timestamp,
                "title": event.title,
                "markdown": event.markdown,
                "meta": event.meta,
                "raw": event.raw,
            })
        );
    }

    #[test]
    fn http_uem_to_egress() {
        let adapter = HttpAdapter;
        let event = UemEvent {
            id: "evt-1".to_string(),
            source: "custom".to_string(),
            timestamp: 1,
            title: None,
            markdown: "hello".to_string(),
            raw: json!({}),
            meta: json!({}),
        };
        let payload = adapter.uem_to_egress(&event).expect("payload");
        assert_yaml_snapshot!(
            "adapters_http_uem_to_egress",
            json!({
                "content_type": payload.content_type,
                "body": payload.body,
            })
        );
    }
}
