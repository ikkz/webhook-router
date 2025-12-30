pub fn extract_title_from_markdown(markdown: &str) -> Option<String> {
    markdown
        .lines()
        .find(|line| !line.trim().is_empty())
        .map(|line| {
            let mut title = line.trim();
            // Remove leading markdown headers
            title = title.trim_start_matches('#').trim();
            
            if title.len() > 100 {
                format!("{}...", &title[..100])
            } else {
                title.to_string()
            }
        })
}
