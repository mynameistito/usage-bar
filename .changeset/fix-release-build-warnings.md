---
"usage-bar-windows": patch
---

Remove unused variable bindings that caused warnings in release builds

The debug-only macros (`debug_amp!`, `debug_error!`) expand to nothing in release builds, leaving their bound variables unused. Removed the `field_name` parameter from `extract_number_optional` and restructured the three `if let Err(e)` patterns in `main.rs` to call `.is_err()` directly, eliminating the bindings entirely.
