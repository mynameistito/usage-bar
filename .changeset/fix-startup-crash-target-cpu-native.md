---
"usage-bar-windows": patch
---

Fix startup crash (STATUS_ILLEGAL_INSTRUCTION) on Intel CPUs caused by `-C target-cpu=native` in `.cargo/config.toml`. GitHub Actions `windows-latest` runners use AMD EPYC CPUs, so the release binary was compiled with AMD SSE4a instructions (e.g. `INSERTQ`) that do not exist on Intel processors. Removing `target-cpu=native` restores the x86-64 baseline and ensures the binary runs on all modern x86-64 CPUs.

Also fix the `CloseRequested` window event handler to call `api.prevent_close()` so that closing the window hides it to the tray instead of destroying it, and fix the frontend error path to call `window.show()` so a startup failure is visible rather than leaving the window permanently hidden.
