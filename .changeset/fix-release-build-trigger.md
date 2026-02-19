---
"usage-bar-windows": patch
---

Fix release pipeline not triggering Windows builds after version PR merge. Inline build jobs directly into the release workflow to bypass GitHub's restriction on cross-workflow event triggers from GITHUB_TOKEN.
