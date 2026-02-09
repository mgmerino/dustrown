use std::{cell::RefCell, env, fs, path::Path, path::PathBuf, rc::Rc};

use gtk::{gdk, prelude::*};
use webkit2gtk::{SettingsExt, WebView, WebViewExt};

use crate::core::{self, AppState};

fn refresh_view(webview: &WebView, state: &AppState) {
    let body = state
        .rendered_html
        .clone()
        .unwrap_or_else(core::default_body);
    let page = core::render_document(&body, state.theme);
    webview.load_html(&page, None);
}

fn open_path(path: &Path, webview: &WebView, window: &gtk::Window, state: &Rc<RefCell<AppState>>) {
    match fs::read_to_string(path) {
        Ok(markdown) => {
            {
                let mut s = state.borrow_mut();
                s.source_markdown = Some(markdown.clone());
                s.rendered_html = Some(core::render_markdown(&markdown, s.theme));
                refresh_view(webview, &s);
            }
            window.set_title(&format!(
                "{} - {}",
                core::APP_TITLE,
                core::filename_or_path(path)
            ));
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
            window.set_title(core::APP_TITLE);
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

pub fn run() {
    if let Err(err) = gtk::init() {
        eprintln!("failed to initialize GTK: {err}");
        return;
    }

    let initial_path = env::args().nth(1).map(PathBuf::from);

    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title(core::APP_TITLE);
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
    if let Some(settings) = WebViewExt::settings(&webview) {
        settings.set_enable_javascript(false);
    }
    scroller.add(&webview);

    vbox.pack_start(&menu_bar, false, false, 0);
    vbox.pack_start(&scroller, true, true, 0);
    window.add(&vbox);

    let state = Rc::new(RefCell::new(AppState {
        theme: core::detect_theme(),
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
            window.set_title(core::APP_TITLE);
        });
    }

    {
        let webview = webview.clone();
        let state = state.clone();
        toggle_theme_item.connect_activate(move |_| {
            let mut s = state.borrow_mut();
            s.theme = s.theme.toggled();

            if let Some(markdown) = s.source_markdown.clone() {
                s.rendered_html = Some(core::render_markdown(&markdown, s.theme));
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
