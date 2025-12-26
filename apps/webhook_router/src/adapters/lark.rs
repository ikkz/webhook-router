use serde_json::{json, Value};
use uuid::Uuid;

use crate::adapters::{AdapterError, WebhookAdapter};
use crate::models::{OutgoingPayload, UemEvent};

#[derive(Debug)]
pub struct LarkAdapter;

impl WebhookAdapter for LarkAdapter {
    fn ingress_to_uem(&self, payload: &Value) -> Result<UemEvent, AdapterError> {
        let header = payload.get("header").unwrap_or(payload);
        let event = payload.get("event").unwrap_or(payload);
        let message = event.get("message").unwrap_or(payload);

        let content_text = message
            .get("content")
            .and_then(|value| value.as_str())
            .and_then(|raw| serde_json::from_str::<Value>(raw).ok())
            .and_then(|value| value.get("text").and_then(|text| text.as_str()).map(String::from))
            .unwrap_or_default();

        let event_id = header
            .get("event_id")
            .and_then(|value| value.as_str())
            .map(String::from)
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let timestamp = header
            .get("event_time")
            .and_then(value_to_i64)
            .map(|millis| millis / 1000)
            .unwrap_or_else(now_timestamp);

        let meta = json!({
            "message_type": message.get("message_type"),
            "chat_id": message.get("chat_id"),
            "sender": event.get("sender"),
        });

        Ok(UemEvent {
            id: event_id,
            source: "lark".to_string(),
            timestamp,
            title: message
                .get("message_type")
                .and_then(|value| value.as_str())
                .map(String::from),
            markdown: if content_text.is_empty() {
                payload.to_string()
            } else {
                content_text
            },
            raw: payload.clone(),
            meta,
        })
    }

    fn uem_to_egress(&self, event: &UemEvent) -> Result<OutgoingPayload, AdapterError> {
        Ok(OutgoingPayload {
            body: json!({
                "msg_type": "text",
                "content": { "text": event.markdown }
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
    use serde_json::json;

    #[test]
    fn lark_ingress_to_uem() {
        let adapter = LarkAdapter;
        let payload = json!({
            "header": { "event_id": "evt-1", "event_time": "1700000000000" },
            "event": {
                "message": {
                    "message_type": "text",
                    "content": "{\"text\":\"hi\"}"
                }
            }
        });
        let event = adapter.ingress_to_uem(&payload).expect("uem");
        assert_eq!(event.id, "evt-1");
        assert_eq!(event.source, "lark");
        assert_eq!(event.markdown, "hi");
    }

    #[test]
    fn lark_uem_to_egress() {
        let adapter = LarkAdapter;
        let event = UemEvent {
            id: "evt-1".to_string(),
            source: "lark".to_string(),
            timestamp: 1,
            title: None,
            markdown: "hello".to_string(),
            raw: json!({}),
            meta: json!({}),
        };
        let payload = adapter.uem_to_egress(&event).expect("payload");
        assert_eq!(payload.body["msg_type"], "text");
        assert_eq!(payload.body["content"]["text"], "hello");
    }
}
