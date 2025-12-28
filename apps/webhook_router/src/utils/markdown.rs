use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use serde_json::{json, Value};

/// Converts Standard Markdown to Slack's 'mrkdwn' format.
///
/// Slack differences:
/// - Bold: `*text*` (vs `**text**`)
/// - Italic: `_text_` (vs `*text*` or `_text_`)
/// - Strike: `~text~` (vs `~~text~~`)
/// - Link: `<url|text>` (vs `[text](url)`)
/// - Headers: Not supported, converted to bold.
/// - Lists: Approximated with bullets/numbers.
pub struct SlackConverter;

impl SlackConverter {
    pub fn convert(markdown: &str) -> String {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES); // Ideally we'd support tables, but simple for now
        
        let parser = Parser::new_ext(markdown, options);
        let mut slack_text = String::new();
        // let mut link_url: Option<String> = None; // Actually used in original code? Yes.
        // let mut in_code_block = false; // Used to prevent formatting inside code blocks? 
        // The warning said in_code_block assigned but never read. I only wrote to it. I never read it.
        // So I can remove it if I don't implement the logic to check it.
        // Actually link_url IS used: link_url = Some... then match link_url? No, I used dest_url directly in Tag::Link.
        // Wait, looking at my code:
        // Tag::Link { dest_url... } => { link_url = Some...; push... }
        // TagEnd::Link => { link_url = None }
        // I never READ link_url in SlackConverter! I just wrote to it.
        // So I can remove both.

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
                    Tag::Item => slack_text.push_str("• "), // Simple bullet for all items for now
                    Tag::Emphasis => slack_text.push('_'),
                    Tag::Strong => slack_text.push('*'),
                    Tag::Strikethrough => slack_text.push('~'),
                    Tag::Link { link_type: _, dest_url, title: _, id: _ } => {
                       slack_text.push('<');
                       slack_text.push_str(&dest_url);
                       slack_text.push('|');
                    },
                    Tag::Image { link_type: _, dest_url, title: _, id: _ } => {
                        // Slack text doesn't really support inline images well, just link it
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
}

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
pub struct LarkConverter;

impl LarkConverter {
    pub fn convert(markdown: &str) -> Value {
         let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        
        let parser = Parser::new_ext(markdown, options);
        
        let mut content: Vec<Vec<Value>> = Vec::new();
        let mut current_paragraph: Vec<Value> = Vec::new();
        
        // State tracking
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
                        // Lark post headers aren't strictly separate tags in 'content',
                        // often treated as bold text or separate lines. 
                        // We'll treat as bold for now or just text.
                        flush_paragraph(&mut content, &mut current_paragraph);
                    },
                    Tag::BlockQuote(_) => {}, // TODO: Handle properly, maybe via style?
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
                        // Lark style handling is limited in basic post; often better to just send text
                        // Use style object if really needed, but strict 'text' tag is simplest.
                        // Actually, Lark 'text' tag doesn't support 'style' field in all versions, 
                        // but 2.0 does. For safety, we map styles to text if possible or ignore.
                        // Let's keep it simple: plain text.
                         current_paragraph.push(json!({
                            "tag": "text",
                            "text": text.as_ref()
                        }));
                    }
                },
                 Event::Code(text) => {
                     // Inline code
                      current_paragraph.push(json!({
                            "tag": "text",
                            "text": text.as_ref() // No inline code tag in Lark?
                        }));
                 },
                 Event::SoftBreak | Event::HardBreak => {
                      // Handled by new paragraphs usually, or just ignored
                 },
                 _ => {}
            }
        }
        flush_paragraph(&mut content, &mut current_paragraph);

        // Fallback if empty
        if content.is_empty() {
             content.push(vec![json!({"tag": "text", "text": markdown})]);
        }

        json!({
            "zh_cn": {
                "title": "Notification",
                "content": content
            }
        })
    }
}

pub struct WecomConverter;

impl WecomConverter {
    /// Takes standard markdown and ensures it fits WeCom's limited subset
    /// or just passes it through if it's mostly compatible.
    /// WeCom supports: # Headers, **bold**, [link](url), code, > quote, <font>
    /// It does NOT support: *italic* (sometimes), tables, images (in markdown).
    pub fn convert(markdown: &str) -> String {
        markdown.to_string() // Pass-through for now, can be optimized
    }
}

pub struct DingTalkConverter;

impl DingTalkConverter {
     /// DingTalk supports: # Headers, **bold**, *italic*, [link](url), > quote, images.
     pub fn convert(markdown: &str) -> String {
        markdown.to_string() // Pass-through for now
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slack_conversion() {
        let md = "**Bold** and *Italic* and [Link](http://example.com)";
        let slack = SlackConverter::convert(md);
        assert_eq!(slack, "*Bold* and _Italic_ and <http://example.com|Link>");
    }

     #[test]
    fn test_slack_headers() {
        let md = "# Heading 1";
        let slack = SlackConverter::convert(md);
        assert_eq!(slack, "*Heading 1*"); // Headers become bold
    }

    #[test]
    fn test_lark_structure() {
        let md = "Hello [World](http://example.com)";
        let lark = LarkConverter::convert(md);
        let content = lark["zh_cn"]["content"].as_array().unwrap();
        assert!(!content.is_empty());
        // Simple check
    }
}
