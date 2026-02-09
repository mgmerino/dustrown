# AGENTS.md

## Project

- Name: `dustrown`
- Type: Rust desktop Markdown viewer
- Goal: dead-simple viewer with HTML rendering, GitHub-like styling, theme toggle, and file actions

## Current Architecture

- Shared core: `src/core.rs`
- Linux backend (GTK3 + WebKit2GTK): `src/linux.rs`
- Windows backend (wry + tao + rfd): `src/windows.rs`
- OS dispatch entrypoint: `src/main.rs`

## Implemented Features

- Open/close/quit actions and keyboard shortcuts
  - `Ctrl+O` open
  - `Ctrl+W` close
  - `Ctrl+D` toggle theme
  - `Ctrl+Q` quit
- CLI file open support (`app /path/file.md`)
- Markdown rendering with fenced code highlighting via `syntect`
- Light/dark mode toggle

## Security Posture

- HTML sanitization with `ammonia`
- JavaScript disabled in Linux WebView
- Security docs:
  - `docs/security-threat-model.md`
  - `docs/malicious-markdown-demo.md`

## Packaging and Local Install

- `Makefile` targets: build/release/install/uninstall/run/clean + desktop/icon helpers
- Default install prefix is user-local: `PREFIX ?= $(HOME)/.local`
- Desktop entry `Exec` uses absolute binary path for launcher compatibility

## CI/CD

- Workflow: `.github/workflows/release.yml`
- Builds Linux + Windows x86_64 artifacts
- Publishes releases on tags matching `v*`

## Branding

- Cargo package name set to `dustrown`
- Icon asset: `docs/icon.svg`

## Recent Git Context

- Previous branches/PRs:
  - `feat/dustrown-rename-packaging` (merged)
  - `feat/github-release-pipeline` (merged)
  - `feat/windows-backend` (PR: https://github.com/mgmerino/dustrown/pull/3)
- Latest fix branch: `fix/windows-error-lifetime`
  - Commit: `7e5c2ef`
  - Fix: Windows compile error `E0716` in `src/windows.rs` by introducing a longer-lived `error_text` binding before `encode_text(...)`

## Current Handoff Status

- Linux build passes locally
- Windows validation is via GitHub Actions
- Next expected actions:
  1. Push `fix/windows-error-lifetime`
  2. Open PR (or move fix into `feat/windows-backend` branch)
  3. Confirm Windows CI green
  4. Merge and optionally tag release `vX.Y.Z`

## Working Preferences

- Keep solutions simple and practical
- Use Rust
- Optimize for Linux/i3 usability
- Prefer minimal-friction, actionable fixes
- When requested, perform branch + commit + PR workflow directly
