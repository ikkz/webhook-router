use serde_json::{json, Value};
use uuid::Uuid;

use crate::adapters::{AdapterError, WebhookAdapter};
use crate::models::{OutgoingPayload, UemEvent};

#[derive(Debug)]
pub struct SlackAdapter;

impl WebhookAdapter for SlackAdapter {
    fn ingress_to_uem(&self, payload: &Value) -> Result<UemEvent, AdapterError> {
        let event = payload.get("event").unwrap_or(payload);
        let text = event
            .get("text")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string();
        let event_id = payload
            .get("event_id")
            .and_then(|value| value.as_str())
            .map(String::from)
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let timestamp = payload
            .get("event_time")
            .and_then(value_to_i64)
            .unwrap_or_else(now_timestamp);

        let meta = json!({
            "channel": event.get("channel"),
            "user": event.get("user"),
            "thread_ts": event.get("thread_ts"),
            "type": event.get("type"),
        });

        Ok(UemEvent {
            id: event_id,
            source: "slack".to_string(),
            timestamp,
            title: event.get("type").and_then(|value| value.as_str()).map(String::from),
            markdown: if text.is_empty() {
                payload.to_string()
            } else {
                text
            },
            raw: payload.clone(),
            meta,
        })
    }

    fn uem_to_egress(&self, event: &UemEvent) -> Result<OutgoingPayload, AdapterError> {
        Ok(OutgoingPayload {
            body: json!({ "text": event.markdown }),
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
    use serde_json::json;

    #[test]
    fn slack_ingress_to_uem() {
        let adapter = SlackAdapter;
        let payload = json!({
            "event_id": "evt-123",
            "event_time": 1700000000,
            "event": { "type": "message", "text": "hello", "channel": "C1" }
        });
        let event = adapter.ingress_to_uem(&payload).expect("uem");
        assert_eq!(event.id, "evt-123");
        assert_eq!(event.source, "slack");
        assert_eq!(event.markdown, "hello");
    }

    #[test]
    fn slack_uem_to_egress() {
        let adapter = SlackAdapter;
        let event = UemEvent {
            id: "evt-1".to_string(),
            source: "slack".to_string(),
            timestamp: 1,
            title: None,
            markdown: "hello".to_string(),
            raw: json!({}),
            meta: json!({}),
        };
        let payload = adapter.uem_to_egress(&event).expect("payload");
        assert_eq!(payload.body["text"], "hello");
    }
}
