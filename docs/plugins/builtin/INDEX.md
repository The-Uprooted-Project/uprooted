# Built-in Plugins

Uprooted ships with built-in plugins across two runtime layers. All are registered automatically at startup and appear in the Plugin Settings page.

---

## Plugin Overview

### TypeScript / Browser Layer

| Plugin | Purpose | Settings |
|--------|---------|----------|
| [Sentry Blocker](sentry-blocker.md) | Blocks Sentry telemetry to protect user privacy | None |
| [Themes](themes.md) | CSS variable theme engine with presets and custom colors | Theme selector, accent/background colors |
| [Link Embeds](link-embeds.md) | Discord-style rich link previews, YouTube thumbnails, animated GIFs | Show file names toggle |
| Silent Typing | Prevents your typing indicator from being sent | None |

### Native Layer

These plugins run natively inside Root's process and modify the UI directly.

| Plugin | Purpose | Settings |
|--------|---------|----------|
| ClearURLs | Strips tracking parameters (utm_*, fbclid, gclid, etc.) from URLs before sending | None |
| [Message Logger](message-logger.md) | Logs deleted and edited messages with visual indicators | Delete/edit toggles, retention limit, ignore own messages |
| Content Filter | Blurs images flagged as NSFW using Google Cloud Vision | API key, threshold |
| [Rootcord](rootcord.md) | Experimental Discord-style vertical server sidebar | Enable toggle |
| Translate | Translate messages and compose in other languages | Language picker, DeepL API key |
| Who Reacted | Shows reactor avatars next to reaction pills | None |
| User Bio | View and edit user bios on profile popups | View-only toggle, bio text |
| Presence Beacon | Uprooted user detection with community badges | None |
| Focus Mode | Hide media, embeds, reactions, and typing indicators for clean reading | Category toggles, placeholder toggle |

### Core Framework (not toggleable)

| Component | Purpose |
|-----------|---------|
| [Settings Panel](settings-panel.md) | Injects Uprooted's settings UI into Root's sidebar |

---

## Runtime Context

### TypeScript / Browser plugins

These run inside Root's embedded Chromium instance. Key constraints:

- **No localStorage**: Root runs Chromium with `--incognito`, so browser storage is wiped on restart
- **Chat is NOT in this context**: Root's chat UI is native Avalonia; the browser handles WebRTC, OAuth, and sub-apps only

### Native plugins

These run inside Root's process. Key characteristics:

- **Settings persist**: stored in `uprooted-settings.ini` in the profile directory
- **Direct UI access**: can create, modify, and inject native controls
- **UI thread required**: all UI mutations must dispatch to the UI thread

## Shared Globals

| Global | Type | Purpose |
|--------|------|---------|
| `window.__UPROOTED_SETTINGS__` | `UprootedSettings` | Settings loaded from `uprooted-settings.json` |
| `window.__UPROOTED_LOADER__` | `PluginLoader` | Plugin lifecycle manager |
| `window.__UPROOTED_VERSION__` | `string` | Version string (e.g. `"0.5.1"`) |
