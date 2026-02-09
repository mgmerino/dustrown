use std::{
    cell::RefCell,
    env, fs,
    path::{Path, PathBuf},
    rc::Rc,
};

use dark_light::Mode;
use gtk::{gdk, prelude::*};
use pulldown_cmark::{html, CodeBlockKind, CowStr, Event, Options, Parser, Tag, TagEnd};
use syntect::{
    highlighting::{Theme as SyntectTheme, ThemeSet},
    html::highlighted_html_for_string,
    parsing::SyntaxSet,
};
use webkit2gtk::{WebView, WebViewExt};

const APP_TITLE: &str = "Markdown Viewer";

#[derive(Copy, Clone)]
enum Theme {
    Light,
    Dark,
}

impl Theme {
    fn toggled(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }
}

struct AppState {
    theme: Theme,
    source_markdown: Option<String>,
    rendered_html: Option<String>,
}

fn detect_theme() -> Theme {
    match dark_light::detect() {
        Mode::Dark => Theme::Dark,
        _ => Theme::Light,
    }
}

fn render_markdown(markdown: &str, theme: Theme) -> String {
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
    let mut output = String::new();
    html::push_html(&mut output, transformed.into_iter());
    output
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

fn default_body() -> String {
    "<p class=\"empty\">Use File -&gt; Open... or Ctrl+O to load a Markdown file.</p>".to_string()
}

fn wrap_html(body: &str, theme: Theme) -> String {
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

fn refresh_view(webview: &WebView, state: &AppState) {
    let body = state.rendered_html.clone().unwrap_or_else(default_body);
    let page = wrap_html(&body, state.theme);
    webview.load_html(&page, None);
}

fn filename_or_path(path: &Path) -> String {
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        name.to_string()
    } else {
        path.to_string_lossy().into_owned()
    }
}

fn open_path(path: &Path, webview: &WebView, window: &gtk::Window, state: &Rc<RefCell<AppState>>) {
    match fs::read_to_string(path) {
        Ok(markdown) => {
            {
                let mut s = state.borrow_mut();
                s.source_markdown = Some(markdown.clone());
                s.rendered_html = Some(render_markdown(&markdown, s.theme));
                refresh_view(webview, &s);
            }
            window.set_title(&format!("{APP_TITLE} - {}", filename_or_path(path)));
        }
        Err(err) => {
            let error_text = err.to_string();
            let escaped = html_escape::encode_text(&error_text);
            {
                let mut s = state.borrow_mut();
                s.source_markdown = None;
                s.rendered_html = Some(format!("<h2>Could not open file</h2><p>{escaped}</p>"));
                refresh_view(webview, &s);
            }
            window.set_title(APP_TITLE);
        }
    }
}

fn open_file_dialog(window: &gtk::Window) -> Option<PathBuf> {
    let dialog = gtk::FileChooserDialog::with_buttons(
        Some("Open Markdown File"),
        Some(window),
        gtk::FileChooserAction::Open,
        &[
            ("Cancel", gtk::ResponseType::Cancel),
            ("Open", gtk::ResponseType::Accept),
        ],
    );

    let filter = gtk::FileFilter::new();
    filter.set_name(Some("Markdown files"));
    filter.add_pattern("*.md");
    filter.add_pattern("*.markdown");
    filter.add_pattern("*.mdown");
    filter.add_pattern("*.mkd");
    filter.add_pattern("*.txt");
    dialog.add_filter(filter);

    let result = if dialog.run() == gtk::ResponseType::Accept {
        dialog.filename()
    } else {
        None
    };

    dialog.close();
    result
}

fn main() {
    if let Err(err) = gtk::init() {
        eprintln!("failed to initialize GTK: {err}");
        return;
    }

    let initial_path = env::args().nth(1).map(PathBuf::from);

    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title(APP_TITLE);
    window.set_default_size(980, 760);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let menu_bar = gtk::MenuBar::new();

    let file_menu_item = gtk::MenuItem::with_label("File");
    let file_menu = gtk::Menu::new();
    let open_item = gtk::MenuItem::with_label("Open...");
    let close_item = gtk::MenuItem::with_label("Close File");
    let quit_item = gtk::MenuItem::with_label("Quit");
    file_menu.append(&open_item);
    file_menu.append(&close_item);
    file_menu.append(&gtk::SeparatorMenuItem::new());
    file_menu.append(&quit_item);
    file_menu_item.set_submenu(Some(&file_menu));

    let view_menu_item = gtk::MenuItem::with_label("View");
    let view_menu = gtk::Menu::new();
    let toggle_theme_item = gtk::MenuItem::with_label("Toggle Light/Dark");
    view_menu.append(&toggle_theme_item);
    view_menu_item.set_submenu(Some(&view_menu));

    menu_bar.append(&file_menu_item);
    menu_bar.append(&view_menu_item);

    let accel_group = gtk::AccelGroup::new();
    window.add_accel_group(&accel_group);
    open_item.add_accelerator(
        "activate",
        &accel_group,
        *gdk::keys::constants::O,
        gdk::ModifierType::CONTROL_MASK,
        gtk::AccelFlags::VISIBLE,
    );
    close_item.add_accelerator(
        "activate",
        &accel_group,
        *gdk::keys::constants::W,
        gdk::ModifierType::CONTROL_MASK,
        gtk::AccelFlags::VISIBLE,
    );
    toggle_theme_item.add_accelerator(
        "activate",
        &accel_group,
        *gdk::keys::constants::D,
        gdk::ModifierType::CONTROL_MASK,
        gtk::AccelFlags::VISIBLE,
    );
    quit_item.add_accelerator(
        "activate",
        &accel_group,
        *gdk::keys::constants::Q,
        gdk::ModifierType::CONTROL_MASK,
        gtk::AccelFlags::VISIBLE,
    );

    let scroller = gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    scroller.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);

    let webview = WebView::new();
    scroller.add(&webview);

    vbox.pack_start(&menu_bar, false, false, 0);
    vbox.pack_start(&scroller, true, true, 0);
    window.add(&vbox);

    let state = Rc::new(RefCell::new(AppState {
        theme: detect_theme(),
        source_markdown: None,
        rendered_html: None,
    }));

    {
        let s = state.borrow();
        refresh_view(&webview, &s);
    }

    if let Some(path) = initial_path {
        open_path(&path, &webview, &window, &state);
    }

    {
        let webview = webview.clone();
        let window = window.clone();
        let state = state.clone();
        open_item.connect_activate(move |_| {
            if let Some(path) = open_file_dialog(&window) {
                open_path(&path, &webview, &window, &state);
            }
        });
    }

    {
        let webview = webview.clone();
        let window = window.clone();
        let state = state.clone();
        close_item.connect_activate(move |_| {
            {
                let mut s = state.borrow_mut();
                s.source_markdown = None;
                s.rendered_html = None;
                refresh_view(&webview, &s);
            }
            window.set_title(APP_TITLE);
        });
    }

    {
        let webview = webview.clone();
        let state = state.clone();
        toggle_theme_item.connect_activate(move |_| {
            let mut s = state.borrow_mut();
            s.theme = s.theme.toggled();

            if let Some(markdown) = s.source_markdown.clone() {
                s.rendered_html = Some(render_markdown(&markdown, s.theme));
            }

            refresh_view(&webview, &s);
        });
    }

    quit_item.connect_activate(|_| gtk::main_quit());
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        gtk::glib::Propagation::Proceed
    });

    window.show_all();
    gtk::main();
}
