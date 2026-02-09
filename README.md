# Markdown Viewer

Dead simple Linux desktop Markdown viewer built with Rust, GTK3, and WebKit2GTK.

## Features

- Open Markdown files from a simple in-window menu bar.
- Open/close/toggle/quit with keyboard shortcuts (works well on i3/minimal WMs).
- Render Markdown as HTML in a desktop window.
- GitHub-inspired Markdown styling.
- Baseline hardening for untrusted files (HTML sanitization + JavaScript disabled).
- Toggle light/dark theme from the menu bar.
- Close currently opened file without quitting the app.

## Build and Run

```bash
cargo run
```

Open a file directly from CLI:

```bash
cargo run -- /path/to/file.md
```

Release binary:

```bash
cargo build --release
./target/release/dustrown
```

Shortcuts:

- `Ctrl+O` open file
- `Ctrl+W` close file
- `Ctrl+D` toggle light/dark
- `Ctrl+Q` quit

## Linux Runtime Requirements

This app uses GTK3 + WebKit2GTK on Linux. Install runtime/dev packages for your distro.

For Debian/Ubuntu-like systems (example package names):

```bash
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev
```

Package names vary by distribution.

## Notes for i3/minimal WMs

Menus are rendered inside the app window and do not depend on desktop/global menu integrations.
