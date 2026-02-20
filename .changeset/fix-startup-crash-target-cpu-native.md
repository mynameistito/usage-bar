---
"usage-bar-windows": patch
---

- Fixed startup crash (STATUS_ILLEGAL_INSTRUCTION) on Intel CPUs caused by `-C target-cpu=native` in `.cargo/config.toml`. GitHub Actions `windows-latest` runners use AMD EPYC CPUs, so the release binary was compiled with AMD SSE4a instructions (e.g. `INSERTQ`) that do not exist on Intel processors. Removing `target-cpu=native` restores the x86-64 baseline and ensures the binary runs on all modern x86-64 CPUs.

- Fixed the `CloseRequested` window event handler to call `api.prevent_close()` so that closing the window hides it to the tray instead of destroying it, and fix the frontend error path to call `window.show()` so a startup failure is visible rather than leaving the window permanently hidden.

- Fixed Tab display bug: Added explicit switchTab("claude") call when no saved tab exists. Ensures tabs are properly hidden on initial load.

- Fixed Z.ai persistence bug: Added "Saving..." state before persisting. The issue was the UI was rebuilding before the save completed, making it appear to work but not actually persisting and by verifying the key/cookie actually exists after saving, before showing "connected" state. Now the UI won't lie if the backend save fails.
