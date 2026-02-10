#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AppEvent {
    Open,
    Close,
    ToggleTheme,
    Quit,
}

pub fn parse_app_event(raw: &str) -> Option<AppEvent> {
    match raw {
        "open" => Some(AppEvent::Open),
        "close" => Some(AppEvent::Close),
        "theme" => Some(AppEvent::ToggleTheme),
        "quit" => Some(AppEvent::Quit),
        _ => None,
    }
}

pub fn extract_document_body(markdown_doc: &str) -> &str {
    let body_start = markdown_doc.find("<body>").map_or(0, |index| index + 6);
    let body_end = markdown_doc.rfind("</body>").unwrap_or(markdown_doc.len());

    if body_start <= body_end && body_end <= markdown_doc.len() {
        &markdown_doc[body_start..body_end]
    } else {
        markdown_doc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_app_event_maps_known_actions() {
        assert_eq!(parse_app_event("open"), Some(AppEvent::Open));
        assert_eq!(parse_app_event("close"), Some(AppEvent::Close));
        assert_eq!(parse_app_event("theme"), Some(AppEvent::ToggleTheme));
        assert_eq!(parse_app_event("quit"), Some(AppEvent::Quit));
        assert_eq!(parse_app_event("unknown"), None);
    }

    #[test]
    fn extract_document_body_returns_inner_body_html() {
        let doc = "<html><head></head><body><article>hello</article></body></html>";

        assert_eq!(extract_document_body(doc), "<article>hello</article>");
    }

    #[test]
    fn extract_document_body_falls_back_to_original_for_malformed_input() {
        let malformed = "<body>missing close";

        assert_eq!(extract_document_body(malformed), "missing close");
    }
}
