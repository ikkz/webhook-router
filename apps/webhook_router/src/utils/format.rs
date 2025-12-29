use anyhow::Result;
use dprint_plugin_markdown::format_text;

/// Formats markdown text using dprint-plugin-markdown
/// This ensures proper spacing, line breaks, and formatting
pub fn format_markdown(text: &str) -> Result<String> {
    use dprint_plugin_markdown::configuration::ConfigurationBuilder;
    
    let config = ConfigurationBuilder::new()
        .line_width(80)
        .build();
    
    // format_text returns Result<Option<String>>
    // None means no changes were needed
    match format_text(text, &config, |_lang, _code, _line_width| Ok(None))? {
        Some(formatted) => Ok(formatted),
        None => Ok(text.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_markdown_adds_spacing() {
        let input = "# Header\nParagraph\n## Subheader\nMore text";
        let result = format_markdown(input).unwrap();
        
        // dprint should add blank lines between sections
        assert!(result.contains("\n\n"));
    }

    #[test]
    fn test_format_markdown_handles_empty() {
        let input = "";
        let result = format_markdown(input).unwrap();
        assert_eq!(result, "");
    }
}
