# Built-in Plugins

Uprooted ships four built-in plugins that run in the DotNetBrowser Chromium layer. They are registered automatically by the plugin loader at startup and appear in the in-app settings panel.

> **Related docs:** [Plugin API Reference](../API_REFERENCE.md) | [Root Environment](../ROOT_ENVIRONMENT.md) | [TypeScript Reference](../../framework/TYPESCRIPT_REFERENCE.md)

---

## Plugin Overview

| Plugin | Purpose | Settings | Source |
|--------|---------|----------|--------|
| [Sentry Blocker](sentry-blocker.md) | Blocks Sentry telemetry to protect user privacy | None | `src/plugins/sentry-blocker/` |
| [Themes](themes.md) | CSS variable theme engine with presets and custom colors | Theme selector, accent/background colors | `src/plugins/themes/` |
| [Settings Panel](settings-panel.md) | Injects Uprooted UI into Root's settings sidebar | None | `src/plugins/settings-panel/` |
| [Link Embeds](link-embeds.md) | Discord-style rich link previews and YouTube embeds | YouTube toggle, website toggle, max embeds | `src/plugins/link-embeds/` |

## Load Order

Plugins are registered and started in this order:

1. **sentry-blocker** -- must run first to block telemetry before Sentry sends anything
2. **themes** -- applies CSS variables before the UI renders
3. **settings-panel** -- depends on the other plugins being registered so it can list them
4. **link-embeds** -- enhances chat content after the page is loaded

## Runtime Context

All built-in plugins run inside DotNetBrowser's embedded Chromium instance. Key constraints:

- **No localStorage** -- Root runs Chromium with `--incognito`, so all browser storage is wiped on restart
- **No CORS restrictions** -- Root runs Chromium with `--disable-web-security`, so fetch works cross-origin
- **Settings are session-only** -- runtime changes via the settings panel reset on restart; use the installer for persistent configuration
- **Chat is NOT in this context** -- Root's chat UI is native Avalonia. DotNetBrowser handles WebRTC, OAuth, and sub-apps only

## Shared Globals

| Global | Type | Purpose |
|--------|------|---------|
| `window.__UPROOTED_SETTINGS__` | `UprootedSettings` | Settings loaded from `uprooted-settings.json` by the installer/patcher |
| `window.__UPROOTED_LOADER__` | `PluginLoader` | Plugin lifecycle manager (used by settings-panel) |
| `window.__UPROOTED_VERSION__` | `string` | Version string (e.g. `"0.3.4"`) |
