# Security Threat Model

This app renders Markdown as HTML in a WebKit view. That is convenient, but it is not safe by default for untrusted files.

## Current risk level

- **Current posture:** unsafe for untrusted Markdown
- **Why:** raw HTML from Markdown can be rendered directly
- **Impact:** JavaScript/HTML payloads may execute in the viewer context

## Why this is possible

- Markdown is converted to HTML and then loaded into a browser engine.
- Many Markdown parsers (including the one used here) allow raw HTML blocks.
- If JavaScript is enabled, `<script>` tags and inline event handlers may run.

## What an attacker can do

- Execute JavaScript inside the app's webview.
- Trigger network requests to attacker-controlled servers.
- Load external resources (images/scripts/styles) as tracking beacons.
- Render deceptive UI elements (fake dialogs or login prompts).

## What is usually not possible directly

- Direct shell execution from JavaScript alone.
- Arbitrary local file reads without a browser/WebKit vulnerability or explicit host bridge.

These are not guarantees. Browser engine vulnerabilities can change the risk profile.

## Practical guidance

- Treat unknown Markdown files as untrusted input.
- Do not open untrusted files with elevated privileges.
- Prefer opening files from known local sources.

## Recommended hardening

1. Sanitize rendered HTML (e.g. with `ammonia`) before loading it.
2. Disable JavaScript for Markdown rendering.
3. Restrict or intercept navigation and external resource loading.
4. Use process sandboxing for defense-in-depth.

## Educational demo

See `docs/malicious-markdown-demo.md` for a harmless local demonstration showing how untrusted Markdown can still execute active web content.
