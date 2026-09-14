# Changelog

## 0.2.0

- `tin status` prints the entry SessionStart delivers, so agents on hosts without
  tin hooks (Claude Code, Grok Build, pi) can use tin directly.
- The Stop threshold now measures context the way the Codex status line shows
  "Context N% used", instead of used/window, which read up to 12000/window
  higher.
- `tin --help` keeps the minimal protocol and links to the INSTALL.md of the same
  version for configuration and the Codex hooks.
