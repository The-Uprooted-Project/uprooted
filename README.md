<p align="center">
  <img src="https://uprooted.sh/og.png" width="700" alt="uprooted" />
</p>

<p align="center">
  a client mod framework for root communications
</p>

<p align="center">
  <a href="https://uprooted.sh"><img src="https://img.shields.io/badge/web-uprooted.sh-2D7D46?style=flat" alt="website" /></a>
  <a href="https://github.com/The-Uprooted-Project/uprooted/releases/latest"><img src="https://img.shields.io/badge/download-latest-2D7D46?style=flat" alt="download" /></a>
  <a href="https://github.com/The-Uprooted-Project/uprooted/releases"><img src="https://img.shields.io/github/downloads/The-Uprooted-Project/uprooted/total?style=flat&color=2D7D46&label=downloads" alt="downloads" /></a>
  <img src="https://img.shields.io/badge/version-0.5.1-2D7D46?style=flat" alt="version" />
  <img src="https://img.shields.io/badge/license-custom-blue?style=flat" alt="license" />
  <img src="https://img.shields.io/badge/platform-windows | linux-lightgrey?style=flat" alt="platform" />
</p>

---

## what is this

Uprooted is a mod framework for [Root Communications](https://rootapp.gg) (like Vencord for Discord). It adds custom UI, themes, and plugins to Root's desktop app at runtime. Two independent layers work together: a native hook for the Avalonia UI and a TypeScript layer for the embedded browser.

## features

### plugins

- **Custom themes** with live preview, HSV color picker, preset themes (Default, Crimson, Loki, Cosmic Smoothie), and custom accent/background colors
- **Link embeds**: Discord-style rich link previews for URLs in chat: YouTube thumbnails, Twitter/X cards, Reddit posts, animated GIF/WebP playback, and any site with OpenGraph or oEmbed support
- **Message logger**: deleted messages stay visible with red styling; edited messages show previous content with an amber indicator
- **Rootcord**: experimental Discord-style vertical server sidebar replacing Root's horizontal tab bar
- **Silent typing**: prevents your typing indicator from being sent
- **ClearURLs**: strips tracking parameters (utm_*, fbclid, gclid, etc.) from URLs before sending
- **Translate**: translate messages and compose in other languages, powered by DeepL
- **Sentry blocker**: blocks telemetry to protect user privacy
- **Content filter**: optional NSFW image detection via Google Cloud Vision
- **Presence beacon**: Uprooted user detection with community badges on profile popups
- **Who Reacted**: shows reactor avatars next to reaction pills
- **User Bio**: view and edit user bios on profile popups
- **Focus Mode**: hide media, embeds, reactions, and typing indicators for a clean reading experience

### framework

- **Settings UI** injected into Root's sidebar: About, Plugin Settings, Theme Settings
- **Live theme preview**: color picker recolors controls during drag at 60fps
- **Self-healing**: automatically re-patches after Root updates
- **Auto-updater**: checks for updates and applies them on next restart
- **Plugin system** with lifecycle hooks, bridge interception, and CSS injection
- **OS notifications** for updates (Windows toast, Linux notify-send)

### installer

- **Console TUI installer** (~600KB single binary, Rust) replacing the old 100MB Tauri GUI
- Automatic Root detection, file deployment
- `--plain` mode for CI and scripting, `--diagnose` mode for troubleshooting
- Linux bash installer with multi-distro support

## install

### windows

Download the latest release from the [releases page](https://github.com/The-Uprooted-Project/uprooted/releases/latest) and run the installer.

### linux

```bash
curl -fsSL https://raw.githubusercontent.com/The-Uprooted-Project/uprooted/main/install-uprooted-linux.sh | bash
```

Or download the installer binary from the [releases page](https://github.com/The-Uprooted-Project/uprooted/releases/latest).

## build

```bash
# TypeScript bundle
pnpm install
pnpm build

# Console TUI installer (Rust)
cd installer/src-tauri && cargo build --release
```

## terms of use

**by using uprooted, you agree to the following:**

1. **do not distribute uprooted or its artifacts** (installers, DLLs, modified binaries) outside of this repository.
2. **do not discuss uprooted in Root's public channels** (official Root servers, Root support, Root social media).
3. violations will result in your UUID being **permanently blacklisted** from uprooted.

these rules exist to protect the project and its users. if you want to share uprooted with someone, send them a link to this repository.

## policy

**uprooted is not affiliated with root communications.** this is an independent community project. all modifications are cosmetic-only and do not interact with root's backend services.

## links

- [uprooted.sh](https://uprooted.sh)
- [download latest release](https://github.com/The-Uprooted-Project/uprooted/releases/latest)
- [uprooted server](https://rootapp.gg/AC0ILwUxgQqJ2MOSMXdGjw)
- admin@watchthelight.org

## license

[uprooted license v1.0](LICENSE) - use pieces with credit, don't redistribute the whole thing
