use std::{env, fs, path::Path, path::PathBuf};

use rfd::FileDialog;
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};
use wry::WebViewBuilder;

use crate::core::{self, AppState};

enum AppEvent {
    Open,
    Close,
    ToggleTheme,
    Quit,
}

fn parse_app_event(raw: &str) -> Option<AppEvent> {
    match raw {
        "open" => Some(AppEvent::Open),
        "close" => Some(AppEvent::Close),
        "theme" => Some(AppEvent::ToggleTheme),
        "quit" => Some(AppEvent::Quit),
        _ => None,
    }
}

fn render_app_shell(content_html: &str, state: &AppState) -> String {
    let body = if content_html.is_empty() {
        core::default_body()
    } else {
        content_html.to_string()
    };
    let markdown_doc = core::render_document(&body, state.theme);

    let body_start = markdown_doc.find("<body>").unwrap_or(0);
    let body_end = markdown_doc.rfind("</body>").unwrap_or(markdown_doc.len());
    let content = &markdown_doc[body_start + 6..body_end];

    format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <style>
      html, body {{ margin: 0; padding: 0; }}
      .topbar {{
        display: flex;
        gap: 8px;
        padding: 10px 12px;
        border-bottom: 1px solid #9ea7b3;
        background: #e9eef6;
        position: sticky;
        top: 0;
        z-index: 2;
      }}
      .topbar button {{
        border: 1px solid #6e7681;
        background: #f6f8fa;
        padding: 5px 10px;
        cursor: pointer;
      }}
      .topbar button:hover {{ background: #e2e8f0; }}
    </style>
  </head>
  <body>
    <nav class="topbar">
      <button onclick="appCmd('open')">Open...</button>
      <button onclick="appCmd('close')">Close File</button>
      <button onclick="appCmd('theme')">Toggle Light/Dark</button>
      <button onclick="appCmd('quit')">Quit</button>
    </nav>
    {content}
    <script>
      function appCmd(action) {{ window.ipc.postMessage(action); }}
      window.addEventListener('keydown', (event) => {{
        if (!event.ctrlKey) return;
        const key = event.key.toLowerCase();
        if (key === 'o') {{ event.preventDefault(); appCmd('open'); }}
        else if (key === 'w') {{ event.preventDefault(); appCmd('close'); }}
        else if (key === 'd') {{ event.preventDefault(); appCmd('theme'); }}
        else if (key === 'q') {{ event.preventDefault(); appCmd('quit'); }}
      }});
    </script>
  </body>
</html>"#
    )
}

fn refresh_view(webview: &wry::WebView, state: &AppState) {
    let content = state
        .rendered_html
        .clone()
        .unwrap_or_else(core::default_body);
    let page = render_app_shell(&content, state);
    let _ = webview.load_html(&page);
}

fn open_path(
    path: &Path,
    webview: &wry::WebView,
    window: &tao::window::Window,
    state: &mut AppState,
) {
    match fs::read_to_string(path) {
        Ok(markdown) => {
            state.source_markdown = Some(markdown.clone());
            state.rendered_html = Some(core::render_markdown(&markdown, state.theme));
            refresh_view(webview, state);
            window.set_title(&format!(
                "{} - {}",
                core::APP_TITLE,
                core::filename_or_path(path)
            ));
        }
        Err(err) => {
            let error_text = err.to_string();
            let escaped = html_escape::encode_text(&error_text);
            state.source_markdown = None;
            state.rendered_html = Some(format!("<h2>Could not open file</h2><p>{escaped}</p>"));
            refresh_view(webview, state);
            window.set_title(core::APP_TITLE);
        }
    }
}

fn open_file_dialog() -> Option<PathBuf> {
    FileDialog::new()
        .add_filter("Markdown", &["md", "markdown", "mdown", "mkd", "txt"])
        .pick_file()
}

pub fn run() {
    let initial_path = env::args().nth(1).map(PathBuf::from);
    let mut state = AppState {
        theme: core::detect_theme(),
        source_markdown: None,
        rendered_html: None,
    };

    let event_loop = EventLoopBuilder::<AppEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_title(core::APP_TITLE)
        .with_inner_size(LogicalSize::new(980.0, 760.0))
        .build(&event_loop)
        .expect("create window");

    let webview = WebViewBuilder::new()
        .with_ipc_handler(move |request| {
            if let Some(event) = parse_app_event(request.body()) {
                let _ = proxy.send_event(event);
            }
        })
        .build(&window)
        .expect("build webview");

    let mut initialized = false;
    let mut pending_initial_path = initial_path;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::MainEventsCleared if !initialized => {
                initialized = true;
                refresh_view(&webview, &state);
                if let Some(path) = pending_initial_path.take() {
                    open_path(&path, &webview, &window, &mut state);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::UserEvent(AppEvent::Open) => {
                if let Some(path) = open_file_dialog() {
                    open_path(&path, &webview, &window, &mut state);
                }
            }
            Event::UserEvent(AppEvent::Close) => {
                state.source_markdown = None;
                state.rendered_html = None;
                refresh_view(&webview, &state);
                window.set_title(core::APP_TITLE);
            }
            Event::UserEvent(AppEvent::ToggleTheme) => {
                state.theme = state.theme.toggled();
                if let Some(markdown) = state.source_markdown.clone() {
                    state.rendered_html = Some(core::render_markdown(&markdown, state.theme));
                }
                refresh_view(&webview, &state);
            }
            Event::UserEvent(AppEvent::Quit) => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}
