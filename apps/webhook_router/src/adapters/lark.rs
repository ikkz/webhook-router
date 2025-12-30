use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::adapters::{AdapterError, WebhookAdapter};
use crate::models::{OutgoingPayload, UemEvent};

/// Converts Standard Markdown to Lark's 'post' JSON structure.
///
/// Lark Rich Text Structure:
/// {
///   "zh_cn": {
///     "title": "Title",
///     "content": [
///       [ {"tag": "text", "text": "foo"} ] // Paragraph 1
///     ]
///   }
/// }
fn markdown_to_lark(title: Option<&str>, markdown: &str) -> Value {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    
    let parser = Parser::new_ext(markdown, options);
    
    let mut content: Vec<Vec<Value>> = Vec::new();
    let mut current_paragraph: Vec<Value> = Vec::new();
    
    // State tracking
    let mut link_url: Option<String> = None;

    let flush_paragraph = |content: &mut Vec<Vec<Value>>, current: &mut Vec<Value>| {
        if !current.is_empty() {
            content.push(current.clone());
            current.clear();
        }
    };

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => {}, // Start collecting
                Tag::Heading { level: _, .. } => {
                    flush_paragraph(&mut content, &mut current_paragraph);
                },
                Tag::BlockQuote(_) => {}, 
                Tag::CodeBlock(_) => {
                     flush_paragraph(&mut content, &mut current_paragraph);
                }, 
                Tag::List(_) => flush_paragraph(&mut content, &mut current_paragraph),
                Tag::Item => {},
                Tag::Emphasis => {},
                Tag::Strong => {},
                Tag::Strikethrough => {},
                Tag::Link { dest_url, .. } => link_url = Some(dest_url.to_string()),
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Paragraph => flush_paragraph(&mut content, &mut current_paragraph),
                TagEnd::Heading(_) => {
                     flush_paragraph(&mut content, &mut current_paragraph);
                },
                TagEnd::BlockQuote(_) => flush_paragraph(&mut content, &mut current_paragraph),
                TagEnd::CodeBlock => flush_paragraph(&mut content, &mut current_paragraph),
                TagEnd::List(_) => flush_paragraph(&mut content, &mut current_paragraph),
                TagEnd::Item => flush_paragraph(&mut content, &mut current_paragraph),
                TagEnd::Emphasis => {},
                TagEnd::Strong => {},
                TagEnd::Strikethrough => {},
                TagEnd::Link => link_url = None,
                _ => {}
            },
            Event::Text(text) => {
                if let Some(url) = &link_url {
                     current_paragraph.push(json!({
                        "tag": "a",
                        "text": text.as_ref(),
                        "href": url
                    }));
                } else {
                     current_paragraph.push(json!({
                        "tag": "text",
                        "text": text.as_ref()
                    }));
                }
            },
             Event::Code(text) => {
                  current_paragraph.push(json!({
                        "tag": "text",
                        "text": text.as_ref()
                    }));
             },
             Event::SoftBreak | Event::HardBreak => {},
             _ => {}
        }
    }
    flush_paragraph(&mut content, &mut current_paragraph);

    // Fallback if empty
    if content.is_empty() {
         content.push(vec![json!({"tag": "text", "text": markdown})]);
    }

    let mut zh_cn = serde_json::Map::new();
    if let Some(t) = title {
        zh_cn.insert("title".to_string(), json!(t));
    }
    zh_cn.insert("content".to_string(), json!(content));

    json!({
        "zh_cn": zh_cn
    })
}



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
            title: None,
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
        let post_content = markdown_to_lark(event.title.as_deref(), &event.markdown);
        Ok(OutgoingPayload {
            body: json!({
                "msg_type": "post",
                "content": { "post": post_content }
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
        assert_yaml_snapshot!(
            "adapters_lark_ingress_text",
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
        assert_yaml_snapshot!(
            "adapters_lark_uem_to_egress",
            json!({
                "content_type": payload.content_type,
                "body": payload.body,
            })
        );
    }

    #[test]
    fn lark_uem_to_egress_with_title() {
        let adapter = LarkAdapter;
        let event = UemEvent {
            id: "evt-2".to_string(),
            source: "lark".to_string(),
            timestamp: 1,
            title: Some("Lark Title".to_string()),
            markdown: "hello".to_string(),
            raw: json!({}),
            meta: json!({}),
        };
        let payload = adapter.uem_to_egress(&event).expect("payload");
        assert_yaml_snapshot!(
            "adapters_lark_uem_to_egress_with_title",
            json!({
                "content_type": payload.content_type,
                "body": payload.body,
            })
        );
    }

    #[test]
    fn test_lark_structure() {
        let md = "Hello [World](http://example.com)";
        let lark = markdown_to_lark(None, md);
        let content = lark["zh_cn"]["content"].as_array().unwrap();
        assert!(!content.is_empty());
    }
}
