---
"usage-bar-windows": patch
---

Fix release CI failing due to lefthook pre-commit hooks running cargo:precheck on Ubuntu, which requires Linux GTK/glib system libraries that are not installed on the runner.
