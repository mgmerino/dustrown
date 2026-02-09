# Security Threat Model

This app renders Markdown as HTML in a WebKit view. That is convenient, and now includes baseline hardening, but untrusted content should still be treated carefully.

## Current risk level

- **Current posture:** hardened, but not fully sandboxed
- **Implemented:** HTML sanitization and JavaScript disabled in WebKit settings
- **Residual risk:** deceptive content, external links/resources, browser engine vulnerabilities

## Implemented mitigations

- Rendered HTML is sanitized with `ammonia` before loading.
- JavaScript execution is disabled in the embedded webview.
- Unsafe attributes/tags and dangerous URI schemes are filtered by the sanitizer.

## What an attacker can do

- Embed deceptive-looking content meant to trick users.
- Include external links/images that leak metadata if opened/loaded.
- Attempt to exploit browser engine bugs in WebKit.

## What is usually not possible directly

- Direct shell execution from JavaScript alone.
- Arbitrary local file reads without a browser/WebKit vulnerability or explicit host bridge.

These are not guarantees. Browser engine vulnerabilities can change the risk profile.

## Practical guidance

- Treat unknown Markdown files as untrusted input.
- Do not open untrusted files with elevated privileges.
- Prefer opening files from known local sources.

## Recommended hardening

1. Restrict or intercept navigation and external resource loading.
2. Optionally disable remote image loading by policy.
3. Use process sandboxing for defense-in-depth.
4. Add an explicit "safe mode" indicator in the UI.

## Educational demo

See `docs/malicious-markdown-demo.md` for a historical/educational payload sample. With current hardening, those payloads should be neutralized.
