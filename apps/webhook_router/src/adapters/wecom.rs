use serde_json::{json, Value};
use uuid::Uuid;

use crate::adapters::{AdapterError, WebhookAdapter};
use crate::models::{OutgoingPayload, UemEvent};

#[derive(Debug)]
pub struct WecomAdapter;

impl WebhookAdapter for WecomAdapter {
    fn ingress_to_uem(&self, payload: &Value) -> Result<UemEvent, AdapterError> {
        let msgtype = payload.get("msgtype").and_then(|value| value.as_str());
        let text = match msgtype {
            Some("markdown") => payload
                .get("markdown")
                .and_then(|value| value.get("content"))
                .and_then(|value| value.as_str())
                .unwrap_or(""),
            Some("markdown_v2") => payload
                .get("markdown_v2")
                .and_then(|value| value.get("content"))
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
            .get("msgid")
            .and_then(|value| value.as_str())
            .map(String::from)
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let timestamp = payload
            .get("create_time")
            .and_then(value_to_i64)
            .unwrap_or_else(now_timestamp);

        Ok(UemEvent {
            id: event_id,
            source: "wecom".to_string(),
            timestamp,
            title: msgtype.map(String::from),
            markdown: if text.is_empty() {
                payload.to_string()
            } else {
                text
            },
            raw: payload.clone(),
            meta: json!({
                "msgtype": payload.get("msgtype"),
                "mentioned_list": payload
                    .get("text")
                    .and_then(|value| value.get("mentioned_list")),
            }),
        })
    }

    fn uem_to_egress(&self, event: &UemEvent) -> Result<OutgoingPayload, AdapterError> {
        let content = if let Some(title) = &event.title {
            format!("# {}\n\n{}", title, event.markdown)
        } else {
            event.markdown.clone()
        };

        Ok(OutgoingPayload {
            body: json!({
                "msgtype": "markdown",
                "markdown": { "content": content }
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
    fn wecom_ingress_to_uem() {
        let adapter = WecomAdapter;
        let payload = json!({
            "msgid": "wx-1",
            "create_time": 1700000000,
            "msgtype": "markdown",
            "markdown": { "content": "hi" }
        });
        let event = adapter.ingress_to_uem(&payload).expect("uem");
        assert_yaml_snapshot!(
            "adapters_wecom_ingress_markdown",
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
    fn wecom_uem_to_egress() {
        let adapter = WecomAdapter;
        let event = UemEvent {
            id: "evt-1".to_string(),
            source: "wecom".to_string(),
            timestamp: 1,
            title: None,
            markdown: "hello".to_string(),
            raw: json!({}),
            meta: json!({}),
        };
        let payload = adapter.uem_to_egress(&event).expect("payload");
        assert_yaml_snapshot!(
            "adapters_wecom_uem_to_egress",
            json!({
                "content_type": payload.content_type,
                "body": payload.body,
            })
        );
    }

    #[test]
    fn wecom_uem_to_egress_with_title() {
        let adapter = WecomAdapter;
        let event = UemEvent {
            id: "evt-2".to_string(),
            source: "wecom".to_string(),
            timestamp: 1,
            title: Some("My Title".to_string()),
            markdown: "hello".to_string(),
            raw: json!({}),
            meta: json!({}),
        };
        let payload = adapter.uem_to_egress(&event).expect("payload");
        assert_yaml_snapshot!(
            "adapters_wecom_uem_to_egress_with_title",
            json!({
                "content_type": payload.content_type,
                "body": payload.body,
            })
        );
    }

    #[test]
    fn wecom_ingress_text() {
        let adapter = WecomAdapter;
        let payload = json!({
            "msgid": "wx-text-1",
            "create_time": 1700000000,
            "msgtype": "text",
            "text": { "content": "hello world" }
        });
        let event = adapter.ingress_to_uem(&payload).expect("uem");
        assert_yaml_snapshot!(
            "adapters_wecom_ingress_text",
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
}
