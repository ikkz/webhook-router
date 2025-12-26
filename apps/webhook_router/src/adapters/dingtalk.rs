use serde_json::{json, Value};
use uuid::Uuid;

use crate::adapters::{AdapterError, WebhookAdapter};
use crate::models::{OutgoingPayload, UemEvent};

#[derive(Debug)]
pub struct DingTalkAdapter;

impl WebhookAdapter for DingTalkAdapter {
    fn ingress_to_uem(&self, payload: &Value) -> Result<UemEvent, AdapterError> {
        let msgtype = payload.get("msgtype").and_then(|value| value.as_str());
        let text = match msgtype {
            Some("markdown") => payload
                .get("markdown")
                .and_then(|value| value.get("text"))
                .and_then(|value| value.as_str())
                .unwrap_or(""),
            _ => payload
                .get("text")
                .and_then(|value| value.get("content"))
                .and_then(|value| value.as_str())
                .unwrap_or(""),
        }
        .to_string();

        let event_id = payload
            .get("msgId")
            .and_then(|value| value.as_str())
            .map(String::from)
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let timestamp = payload
            .get("createAt")
            .and_then(value_to_i64)
            .map(|millis| millis / 1000)
            .unwrap_or_else(now_timestamp);

        Ok(UemEvent {
            id: event_id,
            source: "dingtalk".to_string(),
            timestamp,
            title: payload
                .get("markdown")
                .and_then(|value| value.get("title"))
                .and_then(|value| value.as_str())
                .map(String::from)
                .or_else(|| msgtype.map(String::from)),
            markdown: if text.is_empty() {
                payload.to_string()
            } else {
                text
            },
            raw: payload.clone(),
            meta: json!({
                "msgtype": payload.get("msgtype"),
                "at": payload.get("at"),
            }),
        })
    }

    fn uem_to_egress(&self, event: &UemEvent) -> Result<OutgoingPayload, AdapterError> {
        Ok(OutgoingPayload {
            body: json!({
                "msgtype": "markdown",
                "markdown": {
                    "title": event.title.clone().unwrap_or_else(|| "Webhook Router".to_string()),
                    "text": event.markdown
                }
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
    fn dingtalk_ingress_to_uem() {
        let adapter = DingTalkAdapter;
        let payload = json!({
            "msgId": "ding-1",
            "createAt": 1700000000000_i64,
            "msgtype": "markdown",
            "markdown": { "title": "notice", "text": "hi" }
        });
        let event = adapter.ingress_to_uem(&payload).expect("uem");
        assert_eq!(event.id, "ding-1");
        assert_eq!(event.source, "dingtalk");
        assert_eq!(event.markdown, "hi");
        assert_eq!(event.title.as_deref(), Some("notice"));
    }

    #[test]
    fn dingtalk_uem_to_egress() {
        let adapter = DingTalkAdapter;
        let event = UemEvent {
            id: "evt-1".to_string(),
            source: "dingtalk".to_string(),
            timestamp: 1,
            title: None,
            markdown: "hello".to_string(),
            raw: json!({}),
            meta: json!({}),
        };
        let payload = adapter.uem_to_egress(&event).expect("payload");
        assert_eq!(payload.body["msgtype"], "markdown");
        assert_eq!(payload.body["markdown"]["text"], "hello");
    }
}
