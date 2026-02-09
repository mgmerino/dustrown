use std::path::Path;

use ammonia::Builder as HtmlSanitizer;
use dark_light::Mode;
use pulldown_cmark::{html, CodeBlockKind, CowStr, Event, Options, Parser, Tag, TagEnd};
use syntect::{
    highlighting::{Theme as SyntectTheme, ThemeSet},
    html::highlighted_html_for_string,
    parsing::SyntaxSet,
};

pub const APP_TITLE: &str = "Markdown Viewer";

#[derive(Copy, Clone)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    pub fn toggled(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }
}

pub struct AppState {
    pub theme: Theme,
    pub source_markdown: Option<String>,
    pub rendered_html: Option<String>,
}

pub fn detect_theme() -> Theme {
    match dark_light::detect() {
        Mode::Dark => Theme::Dark,
        _ => Theme::Light,
    }
}

pub fn render_markdown(markdown: &str, theme: Theme) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_FOOTNOTES);

    let parser = Parser::new_ext(markdown, options);
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let themes = ThemeSet::load_defaults();
    let theme_name = match theme {
        Theme::Light => "InspiredGitHub",
        Theme::Dark => "base16-ocean.dark",
    };
    let syntect_theme = themes
        .themes
        .get(theme_name)
        .or_else(|| themes.themes.values().next())
        .expect("syntect theme available");

    let transformed = inject_highlighted_code_blocks(parser, &syntax_set, syntect_theme);
    let mut rendered = String::new();
    html::push_html(&mut rendered, transformed.into_iter());

    sanitize_rendered_html(&rendered)
}

pub fn default_body() -> String {
    "<p class=\"empty\">Use File -&gt; Open... or Ctrl+O to load a Markdown file.</p>".to_string()
}

pub fn render_document(body: &str, theme: Theme) -> String {
    let (bg, fg, muted, border, code_bg, quote, link) = match theme {
        Theme::Light => (
            "#ffffff", "#1f2328", "#57606a", "#d0d7de", "#f6f8fa", "#656d76", "#0969da",
        ),
        Theme::Dark => (
            "#0d1117", "#e6edf3", "#9198a1", "#30363d", "#161b22", "#8b949e", "#4493f8",
        ),
    };

    format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <style>
      html, body {{
        margin: 0;
        padding: 0;
        background: {bg};
        color: {fg};
        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", "Noto Sans", Helvetica, Arial, sans-serif;
      }}
      .markdown-body {{
        box-sizing: border-box;
        max-width: 980px;
        margin: 0 auto;
        padding: 32px;
        line-height: 1.6;
        word-wrap: break-word;
      }}
      @media (max-width: 767px) {{ .markdown-body {{ padding: 18px; }} }}
      .empty {{ color: {muted}; }}
      .markdown-body h1, .markdown-body h2, .markdown-body h3,
      .markdown-body h4, .markdown-body h5, .markdown-body h6 {{
        margin-top: 24px;
        margin-bottom: 16px;
        font-weight: 600;
        line-height: 1.25;
        border-bottom: 1px solid transparent;
      }}
      .markdown-body h1, .markdown-body h2 {{
        padding-bottom: 0.3em;
        border-bottom-color: {border};
      }}
      .markdown-body p, .markdown-body ul, .markdown-body ol,
      .markdown-body table, .markdown-body pre, .markdown-body blockquote {{
        margin-top: 0;
        margin-bottom: 16px;
      }}
      .markdown-body a {{ color: {link}; text-decoration: none; }}
      .markdown-body a:hover {{ text-decoration: underline; }}
      .markdown-body code, .markdown-body tt {{
        padding: 0.2em 0.4em;
        margin: 0;
        font-size: 85%;
        border-radius: 6px;
        background: {code_bg};
      }}
      .markdown-body pre {{
        padding: 16px;
        overflow: auto;
        border-radius: 8px;
        background: {code_bg};
      }}
      .markdown-body pre code {{ padding: 0; background: transparent; border-radius: 0; }}
      .markdown-body blockquote {{
        padding: 0 1em;
        color: {quote};
        border-left: 0.25em solid {border};
      }}
      .markdown-body table {{
        display: block;
        width: max-content;
        max-width: 100%;
        overflow: auto;
        border-collapse: collapse;
      }}
      .markdown-body table th, .markdown-body table td {{
        padding: 6px 13px;
        border: 1px solid {border};
      }}
      .markdown-body hr {{
        height: 0.25em;
        margin: 24px 0;
        background: {border};
        border: 0;
      }}
      .markdown-body img {{ max-width: 100%; height: auto; }}
    </style>
  </head>
  <body>
    <article class="markdown-body">{body}</article>
  </body>
</html>"#
    )
}

pub fn filename_or_path(path: &Path) -> String {
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        name.to_string()
    } else {
        path.to_string_lossy().into_owned()
    }
}

fn sanitize_rendered_html(dirty_html: &str) -> String {
    let mut sanitizer = HtmlSanitizer::default();
    sanitizer.add_tag_attributes("a", &["href", "title"]);
    sanitizer.add_tag_attributes("img", &["src", "alt", "title"]);
    sanitizer.add_tag_attributes("code", &["class"]);
    sanitizer.add_tag_attributes("pre", &["class", "style"]);
    sanitizer.add_tag_attributes("span", &["class", "style"]);

    sanitizer.clean(dirty_html).to_string()
}

fn inject_highlighted_code_blocks<'a>(
    parser: Parser<'a>,
    syntax_set: &SyntaxSet,
    syntect_theme: &SyntectTheme,
) -> Vec<Event<'a>> {
    let mut output = Vec::new();
    let mut iter = parser.into_iter();

    while let Some(event) = iter.next() {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                let language = match kind {
                    CodeBlockKind::Fenced(info) => info
                        .split_whitespace()
                        .next()
                        .map(|s| s.trim_start_matches("language-").to_string()),
                    CodeBlockKind::Indented => None,
                };

                let mut code = String::new();
                for code_event in iter.by_ref() {
                    match code_event {
                        Event::End(TagEnd::CodeBlock) => break,
                        Event::Text(text) | Event::Code(text) | Event::Html(text) => {
                            code.push_str(&text)
                        }
                        Event::SoftBreak | Event::HardBreak => code.push('\n'),
                        _ => {}
                    }
                }

                let highlighted =
                    highlight_code_block(&code, language.as_deref(), syntax_set, syntect_theme);
                output.push(Event::Html(CowStr::from(highlighted)));
            }
            other => output.push(other),
        }
    }

    output
}

fn highlight_code_block(
    code: &str,
    language: Option<&str>,
    syntax_set: &SyntaxSet,
    syntect_theme: &SyntectTheme,
) -> String {
    let syntax = language
        .and_then(|lang| syntax_set.find_syntax_by_token(lang))
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());

    highlighted_html_for_string(code, syntax_set, syntax, syntect_theme).unwrap_or_else(|_| {
        let escaped = html_escape::encode_text(code);
        format!("<pre><code>{escaped}</code></pre>")
    })
}
