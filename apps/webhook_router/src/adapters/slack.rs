use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::adapters::{AdapterError, WebhookAdapter};
use crate::models::{OutgoingPayload, UemEvent};

/// Converts Standard Markdown to Slack's 'mrkdwn' format.
fn markdown_to_slack(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES); 
    
    let parser = Parser::new_ext(markdown, options);
    let mut slack_text = String::new();

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => {},
                Tag::Heading { .. } => slack_text.push('*'),
                Tag::BlockQuote(_) => slack_text.push_str("> "),
                Tag::CodeBlock(_) => {
                    slack_text.push_str("```\n");
                }
                Tag::List(Some(_)) => {}, // Ordered list
                Tag::List(None) => {},    // Unordered list
                Tag::Item => slack_text.push_str("• "),
                Tag::Emphasis => slack_text.push('_'),
                Tag::Strong => slack_text.push('*'),
                Tag::Strikethrough => slack_text.push('~'),
                Tag::Link { link_type: _, dest_url, title: _, id: _ } => {
                   slack_text.push('<');
                   slack_text.push_str(&dest_url);
                   slack_text.push('|');
                },
                Tag::Image { link_type: _, dest_url, title: _, id: _ } => {
                     slack_text.push('<');
                     slack_text.push_str(&dest_url);
                     slack_text.push('|');
                }
                 _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Paragraph => slack_text.push('\n'),
                TagEnd::Heading(_) => slack_text.push_str("*\n"),
                TagEnd::BlockQuote(_) => slack_text.push('\n'),
                TagEnd::CodeBlock => {
                    slack_text.push_str("\n```\n");
                }
                TagEnd::List(_) => {},
                TagEnd::Item => slack_text.push('\n'),
                TagEnd::Emphasis => slack_text.push('_'),
                TagEnd::Strong => slack_text.push('*'),
                TagEnd::Strikethrough => slack_text.push('~'),
                TagEnd::Link => {
                     slack_text.push('>');
                },
                TagEnd::Image => {
                    slack_text.push_str("image>");
                },
                 _ => {}
            },
            Event::Text(text) => {
                slack_text.push_str(&text);
            },
            Event::Code(text) => {
               slack_text.push('`');
               slack_text.push_str(&text);
               slack_text.push('`');
            }
            Event::SoftBreak => slack_text.push('\n'),
            Event::HardBreak => slack_text.push('\n'),
            Event::Rule => slack_text.push_str("---\n"),
            _ => {}
        }
    }
    
    slack_text.trim().to_string()
}

#[derive(Debug)]
pub struct SlackAdapter;

impl WebhookAdapter for SlackAdapter {
    fn ingress_to_uem(&self, payload: &Value) -> Result<UemEvent, AdapterError> {
        let event = payload.get("event").unwrap_or(payload);
        let text = slack_markdown_from_payload(event);
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
            title: None,
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
        let markdown = if let Some(title) = &event.title {
            format!("# {}\n\n{}", title, event.markdown)
        } else {
            event.markdown.clone()
        };
        let mrkdwn = markdown_to_slack(&markdown);
        Ok(OutgoingPayload {
            body: json!({ "text": mrkdwn }),
            content_type: "application/json",
        })
    }
}

fn slack_markdown_from_payload(event: &Value) -> String {
    let mut parts = Vec::new();

    let text = event
        .get("text")
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .to_string();

    if let Some(blocks) = event.get("blocks").and_then(|value| value.as_array()) {
        let mut block_parts = Vec::new();
        for block in blocks {
            if let Some(block_text) = slack_block_to_markdown(block) {
                if !block_text.is_empty() {
                    block_parts.push(block_text);
                }
            }
        }
        if !block_parts.is_empty() {
            parts.push(block_parts.join("\n\n"));
        }
    } else if !text.is_empty() {
        parts.push(text);
    }

    if let Some(attachments) = event.get("attachments").and_then(|value| value.as_array()) {
        let mut attachment_parts = Vec::new();
        for attachment in attachments {
            let attachment_text = slack_attachment_to_markdown(attachment);
            if !attachment_text.is_empty() {
                attachment_parts.push(attachment_text);
            }
        }
        if !attachment_parts.is_empty() {
            parts.push(attachment_parts.join("\n\n"));
        }
    }

    if let Some(sections) = event.get("sections").and_then(|value| value.as_array()) {
        let mut section_parts = Vec::new();
        for section in sections {
            let title = section
                .get("activityTitle")
                .and_then(|value| value.as_str())
                .unwrap_or("");
            let subtitle = section
                .get("activitySubtitle")
                .and_then(|value| value.as_str())
                .unwrap_or("");
            let mut lines = Vec::new();
            if !title.is_empty() {
                lines.push(format!("**{}**", title));
            }
            if !subtitle.is_empty() {
                lines.push(subtitle.to_string());
            }
            if !lines.is_empty() {
                section_parts.push(lines.join("\n"));
            }
        }
        if !section_parts.is_empty() {
            parts.push(section_parts.join("\n\n"));
        }
    }

    if parts.is_empty() {
        event.to_string()
    } else {
        parts.join("\n\n")
    }
}

fn slack_block_to_markdown(block: &Value) -> Option<String> {
    let block_type = block.get("type").and_then(|value| value.as_str()).unwrap_or("");
    match block_type {
        "section" => {
            let mut lines = Vec::new();
            if let Some(text) = slack_text_object_text(block.get("text")) {
                lines.push(text);
            }
            if let Some(fields) = block.get("fields").and_then(|value| value.as_array()) {
                for field in fields {
                    if let Some(text) = slack_text_object_text(Some(field)) {
                        lines.push(format!("- {}", text));
                    }
                }
            }
            if lines.is_empty() {
                None
            } else {
                Some(lines.join("\n"))
            }
        }
        "header" => slack_text_object_text(block.get("text")).map(|text| format!("**{}**", text)),
        "divider" => Some("---".to_string()),
        "image" => {
            let alt_text = block
                .get("alt_text")
                .and_then(|value| value.as_str())
                .unwrap_or("image");
            let url = block
                .get("image_url")
                .and_then(|value| value.as_str())
                .unwrap_or("");
            if url.is_empty() {
                None
            } else {
                Some(format!("![{}]({})", alt_text, url))
            }
        }
        "context" => {
            let mut lines = Vec::new();
            if let Some(elements) = block.get("elements").and_then(|value| value.as_array()) {
                for element in elements {
                    if let Some(text) = slack_text_object_text(Some(element)) {
                        lines.push(text);
                    } else if let Some(url) =
                        element.get("image_url").and_then(|value| value.as_str())
                    {
                        lines.push(format!("![image]({})", url));
                    }
                }
            }
            if lines.is_empty() {
                None
            } else {
                Some(lines.join(" "))
            }
        }
        "actions" => {
            let mut lines = Vec::new();
            if let Some(elements) = block.get("elements").and_then(|value| value.as_array()) {
                for element in elements {
                    if element.get("type").and_then(|value| value.as_str()) == Some("button") {
                        let label = slack_text_object_text(element.get("text")).unwrap_or_default();
                        let url = element
                            .get("url")
                            .and_then(|value| value.as_str())
                            .unwrap_or("");
                        if !label.is_empty() && !url.is_empty() {
                            lines.push(format!("- [{}]({})", label, url));
                        }
                    }
                }
            }
            if lines.is_empty() {
                None
            } else {
                Some(lines.join("\n"))
            }
        }
        "rich_text" => slack_rich_text_to_markdown(block),
        _ => None,
    }
}

fn slack_rich_text_to_markdown(block: &Value) -> Option<String> {
    let elements = block.get("elements").and_then(|value| value.as_array())?;
    let mut lines = Vec::new();
    for element in elements {
        if element.get("type").and_then(|value| value.as_str()) == Some("rich_text_section") {
            if let Some(section_elements) = element.get("elements").and_then(|value| value.as_array())
            {
                let mut text = String::new();
                for item in section_elements {
                    match item.get("type").and_then(|value| value.as_str()) {
                        Some("text") => {
                            if let Some(value) = item.get("text").and_then(|value| value.as_str())
                            {
                                text.push_str(value);
                            }
                        }
                        Some("link") => {
                            let url = item.get("url").and_then(|value| value.as_str()).unwrap_or("");
                            let label = item
                                .get("text")
                                .and_then(|value| value.as_str())
                                .unwrap_or(url);
                            if !url.is_empty() {
                                text.push_str(&format!("[{}]({})", label, url));
                            }
                        }
                        Some("emoji") => {
                            if let Some(name) = item.get("name").and_then(|value| value.as_str())
                            {
                                text.push_str(&format!(":{}:", name));
                            }
                        }
                        _ => {}
                    }
                }
                if !text.is_empty() {
                    lines.push(text);
                }
            }
        }
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

fn slack_attachment_to_markdown(attachment: &Value) -> String {
    let mut parts = Vec::new();
    let title = attachment
        .get("title")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    let title_link = attachment
        .get("title_link")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    if !title.is_empty() {
        if !title_link.is_empty() {
            parts.push(format!("[{}]({})", title, title_link));
        } else {
            parts.push(format!("**{}**", title));
        }
    }

    if let Some(text) = attachment.get("text").and_then(|value| value.as_str()) {
        if !text.is_empty() {
            parts.push(text.to_string());
        }
    }

    if let Some(fields) = attachment.get("fields").and_then(|value| value.as_array()) {
        let mut lines = Vec::new();
        for field in fields {
            let title = field
                .get("title")
                .and_then(|value| value.as_str())
                .unwrap_or("");
            let value = field
                .get("value")
                .and_then(|value| value.as_str())
                .unwrap_or("");
            if !title.is_empty() && !value.is_empty() {
                lines.push(format!("- **{}**: {}", title, value));
            }
        }
        if !lines.is_empty() {
            parts.push(lines.join("\n"));
        }
    }

    if let Some(url) = attachment
        .get("image_url")
        .and_then(|value| value.as_str())
    {
        if !url.is_empty() {
            parts.push(format!("![image]({})", url));
        }
    }

    parts.join("\n\n")
}

fn slack_text_object_text(value: Option<&Value>) -> Option<String> {
    value
        .and_then(|value| value.get("text"))
        .and_then(|value| value.as_str())
        .map(String::from)
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
    fn slack_ingress_to_uem() {
        let adapter = SlackAdapter;
        let payload = json!({
            "event_id": "evt-123",
            "event_time": 1700000000,
            "event": { "type": "message", "text": "hello", "channel": "C1" }
        });
        let event = adapter.ingress_to_uem(&payload).expect("uem");
        assert_yaml_snapshot!(
            "adapters_slack_ingress_basic",
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
    fn slack_uem_to_egress() {
        let adapter = SlackAdapter;
        let event = UemEvent {
            id: "evt-1".to_string(),
            source: "slack".to_string(),
            timestamp: 1,
            title: None,
            markdown: "**bold**".to_string(),
            raw: json!({}),
            meta: json!({}),
        };
        let payload = adapter.uem_to_egress(&event).expect("payload");
        assert_yaml_snapshot!(
            "adapters_slack_uem_to_egress",
            json!({
                "content_type": payload.content_type,
                "body": payload.body,
            })
        );
    }

    #[test]
    fn slack_uem_to_egress_with_title() {
        let adapter = SlackAdapter;
        let event = UemEvent {
            id: "evt-2".to_string(),
            source: "slack".to_string(),
            timestamp: 1,
            title: Some("My Header".to_string()),
            markdown: "hello".to_string(),
            raw: json!({}),
            meta: json!({}),
        };
        let payload = adapter.uem_to_egress(&event).expect("payload");
        assert_yaml_snapshot!(
            "adapters_slack_uem_to_egress_with_title",
            json!({
                "content_type": payload.content_type,
                "body": payload.body,
            })
        );
    }

    #[test]
    fn slack_ingress_with_attachments() {
        let adapter = SlackAdapter;
        let payload = json!({
            "event_id": "evt-attach-1",
            "event_time": 1700000000,
            "alias": "GlitchTip",
            "attachments": [
                {
                    "color": "#e52b50",
                    "fields": [
                        { "short": true, "title": "Project", "value": "platform-fe" },
                        { "short": true, "title": "Environment", "value": "production" },
                        { "short": false, "title": "Release", "value": "platform-fe@production-1add165" }
                    ],
                    "image_url": null,
                    "mrkdown_in": ["text"],
                    "text": null,
                    "title": "Error: test",
                    "title_link": "https://example.com"
                }
            ],
            "sections": [
                {
                    "activitySubtitle": "[View Issue PLATFORM-FE-17](https://example.com)",
                    "activityTitle": "Error: test"
                }
            ],
            "text": "GlitchTip Alert"
        });
        let event = adapter.ingress_to_uem(&payload).expect("uem");
        assert_yaml_snapshot!(
            "adapters_slack_ingress_attachments",
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
    fn test_slack_conversion() {
        let md = "**Bold** and *Italic* and [Link](http://example.com)";
        let slack = markdown_to_slack(md);
        assert_eq!(slack, "*Bold* and _Italic_ and <http://example.com|Link>");
    }

     #[test]
    fn test_slack_headers() {
        let md = "# Heading 1";
        let slack = markdown_to_slack(md);
        assert_eq!(slack, "*Heading 1*"); // Headers become bold
    }
}
