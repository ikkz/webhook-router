use serde_json::{json, Value};
use uuid::Uuid;

use crate::adapters::{AdapterError, WebhookAdapter};
use crate::models::{OutgoingPayload, UemEvent};
use crate::utils::markdown::extract_title_from_markdown;

#[derive(Debug)]
pub struct DingTalkAdapter;

impl WebhookAdapter for DingTalkAdapter {
    fn ingress_to_uem(&self, payload: &Value) -> Result<UemEvent, AdapterError> {
        let msgtype = payload.get("msgtype").and_then(|value| value.as_str());
        let text = dingtalk_markdown_from_payload(payload, msgtype);
        let title = dingtalk_title_from_payload(payload, msgtype);

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
            title,
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
        let (title, text) = if let Some(t) = &event.title {
            (t.clone(), format!("# {}\n\n{}", t, event.markdown))
        } else {
            (
                extract_title_from_markdown(&event.markdown)
                    .unwrap_or_else(|| "Webhook Router".to_string()),
                event.markdown.clone(),
            )
        };

        Ok(OutgoingPayload {
            body: json!({
                "msgtype": "markdown",
                "markdown": {
                    "title": title,
                    "text": text
                }
            }),
            content_type: "application/json",
        })
    }
}

fn dingtalk_markdown_from_payload(payload: &Value, msgtype: Option<&str>) -> String {
    match msgtype {
        Some("text") => payload
            .get("text")
            .and_then(|value| value.get("content"))
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string(),
        Some("markdown") => payload
            .get("markdown")
            .and_then(|value| value.get("text"))
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string(),
        Some("link") => {
            let link = payload.get("link").unwrap_or(&Value::Null);
            let title = link
                .get("title")
                .and_then(|value| value.as_str())
                .unwrap_or("");
            let text = link
                .get("text")
                .and_then(|value| value.as_str())
                .unwrap_or("");
            let url = link
                .get("messageUrl")
                .and_then(|value| value.as_str())
                .unwrap_or("");
            let pic_url = link
                .get("picUrl")
                .and_then(|value| value.as_str())
                .unwrap_or("");
            let mut parts = Vec::new();
            if !title.is_empty() {
                parts.push(format!("**{}**", title));
            }
            if !text.is_empty() {
                parts.push(text.to_string());
            }
            if !url.is_empty() {
                parts.push(format!("[{}]({})", if title.is_empty() { "Link" } else { title }, url));
            }
            if !pic_url.is_empty() {
                parts.push(format!("![image]({})", pic_url));
            }
            parts.join("\n\n")
        }
        Some("actionCard") => {
            let action = payload.get("actionCard").unwrap_or(&Value::Null);
            let title = action
                .get("title")
                .and_then(|value| value.as_str())
                .unwrap_or("");
            let text = action
                .get("text")
                .and_then(|value| value.as_str())
                .unwrap_or("");
            let single_title = action
                .get("singleTitle")
                .and_then(|value| value.as_str())
                .unwrap_or("");
            let single_url = action
                .get("singleURL")
                .and_then(|value| value.as_str())
                .unwrap_or("");
            let mut parts = Vec::new();
            if !title.is_empty() {
                parts.push(format!("## {}", title));
            }
            if !text.is_empty() {
                parts.push(text.to_string());
            }
            if !single_title.is_empty() && !single_url.is_empty() {
                parts.push(format!("[{}]({})", single_title, single_url));
            }
            if let Some(buttons) = action.get("btns").and_then(|value| value.as_array()) {
                let mut lines = Vec::new();
                for button in buttons {
                    let label = button
                        .get("title")
                        .and_then(|value| value.as_str())
                        .unwrap_or("");
                    let url = button
                        .get("actionURL")
                        .and_then(|value| value.as_str())
                        .unwrap_or("");
                    if !label.is_empty() && !url.is_empty() {
                        lines.push(format!("- [{}]({})", label, url));
                    }
                }
                if !lines.is_empty() {
                    parts.push(lines.join("\n"));
                }
            }
            parts.join("\n\n")
        }
        Some("feedCard") => {
            let feed = payload.get("feedCard").unwrap_or(&Value::Null);
            if let Some(links) = feed.get("links").and_then(|value| value.as_array()) {
                let mut items = Vec::new();
                for link in links {
                    let title = link
                        .get("title")
                        .and_then(|value| value.as_str())
                        .unwrap_or("");
                    let url = link
                        .get("messageURL")
                        .and_then(|value| value.as_str())
                        .unwrap_or("");
                    let pic_url = link
                        .get("picURL")
                        .and_then(|value| value.as_str())
                        .unwrap_or("");
                    if !title.is_empty() && !url.is_empty() {
                        items.push(format!("- [{}]({})", title, url));
                    }
                    if !pic_url.is_empty() {
                        items.push(format!("  ![image]({})", pic_url));
                    }
                }
                return items.join("\n");
            }
            String::new()
        }
        _ => payload
            .get("text")
            .and_then(|value| value.get("content"))
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string(),
    }
}

fn dingtalk_title_from_payload(payload: &Value, msgtype: Option<&str>) -> Option<String> {
    match msgtype {
        Some("markdown") => payload
            .get("markdown")
            .and_then(|value| value.get("title"))
            .and_then(|value| value.as_str())
            .map(String::from),
        Some("link") => payload
            .get("link")
            .and_then(|value| value.get("title"))
            .and_then(|value| value.as_str())
            .map(String::from),
        Some("actionCard") => payload
            .get("actionCard")
            .and_then(|value| value.get("title"))
            .and_then(|value| value.as_str())
            .map(String::from),
        Some("feedCard") => payload
            .get("feedCard")
            .and_then(|value| value.get("links"))
            .and_then(|value| value.as_array())
            .and_then(|links| {
                links
                    .first()
                    .and_then(|link| link.get("title"))
                    .and_then(|value| value.as_str())
            })
            .map(String::from),
        _ => None,
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
    fn dingtalk_ingress_to_uem() {
        let adapter = DingTalkAdapter;
        let payload = json!({
            "msgId": "ding-1",
            "createAt": 1700000000000_i64,
            "msgtype": "markdown",
            "markdown": { "title": "notice", "text": "hi" }
        });
        let event = adapter.ingress_to_uem(&payload).expect("uem");
        assert_yaml_snapshot!(
            "adapters_dingtalk_ingress_markdown",
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
    fn dingtalk_ingress_text() {
        let adapter = DingTalkAdapter;
        let payload = json!({
            "msgId": "ding-text-1",
            "createAt": 1700000000000_i64,
            "msgtype": "text",
            "text": { "content": "我就是我, @180xxxxxx 是不一样的烟火" },
            "at": { "atMobiles": ["180xxxxxx"], "isAtAll": false }
        });
        let event = adapter.ingress_to_uem(&payload).expect("uem");
        assert_yaml_snapshot!(
            "adapters_dingtalk_ingress_text",
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
    fn dingtalk_ingress_link() {
        let adapter = DingTalkAdapter;
        let payload = json!({
            "msgId": "ding-link-1",
            "createAt": 1700000000000_i64,
            "msgtype": "link",
            "link": {
                "text": "这是Link消息",
                "title": "这是一个Link消息",
                "picUrl": "https://img.alicdn.com/tfs/TB1NwmBEL9TBuNjy1zbXXXpepXa-2400-1218.png",
                "messageUrl": "https://open.dingtalk.com/document/"
            }
        });
        let event = adapter.ingress_to_uem(&payload).expect("uem");
        assert_yaml_snapshot!(
            "adapters_dingtalk_ingress_link",
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
    fn dingtalk_ingress_action_card() {
        let adapter = DingTalkAdapter;
        let payload = json!({
            "msgId": "ding-action-1",
            "createAt": 1700000000000_i64,
            "msgtype": "actionCard",
            "actionCard": {
                "title": "整体跳转actionCard消息",
                "text": "![这是一张图片](https://img.alicdn.com/tfs/TB1NwmBEL9TBuNjy1zbXXXpepXa-2400-1218.png) \n  这是一个整体跳转actionCard消息",
                "singleTitle" : "阅读全文",
                "singleURL" : "https://open.dingtalk.com/document/"
            }
        });
        let event = adapter.ingress_to_uem(&payload).expect("uem");
        assert_yaml_snapshot!(
            "adapters_dingtalk_ingress_action_card",
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
    fn dingtalk_ingress_feed_card() {
        let adapter = DingTalkAdapter;
        let payload = json!({
            "msgId": "ding-feed-1",
            "createAt": 1700000000000_i64,
            "msgtype": "feedCard",
            "feedCard": {
                "links": [
                    {
                        "title": "这是feedcard消息1",
                        "messageURL": "https://open.dingtalk.com/document/",
                        "picURL": "https://img.alicdn.com/tfs/TB1NwmBEL9TBuNjy1zbXXXpepXa-2400-1218.png"
                    }
                ]
            }
        });
        let event = adapter.ingress_to_uem(&payload).expect("uem");
        assert_yaml_snapshot!(
            "adapters_dingtalk_ingress_feed_card",
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
        assert_yaml_snapshot!(
            "adapters_dingtalk_uem_to_egress",
            json!({
                "content_type": payload.content_type,
                "body": payload.body,
            })
        );
    }

    #[test]
    fn dingtalk_uem_to_egress_with_title() {
        let adapter = DingTalkAdapter;
        let event = UemEvent {
            id: "evt-2".to_string(),
            source: "dingtalk".to_string(),
            timestamp: 1,
            title: Some("My Title".to_string()),
            markdown: "hello".to_string(),
            raw: json!({}),
            meta: json!({}),
        };
        let payload = adapter.uem_to_egress(&event).expect("payload");
        assert_yaml_snapshot!(
            "adapters_dingtalk_uem_to_egress_with_title",
            json!({
                "content_type": payload.content_type,
                "body": payload.body,
            })
        );
    }
}
