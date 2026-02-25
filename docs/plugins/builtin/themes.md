# Themes

> **What this is:** Themes plugin reference — CSS variable theme engine, color math, custom theme generation.

Built-in theme engine that overrides Root's CSS variable color system. Supports preset themes and custom accent + background color generation.

> **Source:** [`src/plugins/themes/index.ts`](../../../src/plugins/themes/index.ts), [`src/plugins/themes/themes.json`](../../../src/plugins/themes/themes.json)

---

## What it does

Root's **web-side** color system is driven by `--rootsdk-*` CSS variables on `document.documentElement`. This plugin overrides those variables to apply custom color schemes across the Chromium/web UI. The native Avalonia UI (chat, sidebar, settings) uses a separate theme engine that overrides native color keys.

> **Dual-Layer Architecture:** This plugin controls the Chromium/web layer only. A separate native theme engine independently controls the Avalonia layer. Both systems coordinate to keep the web and native sides in sync when the user changes themes.

Three modes:
- **Default** -- no overrides, Root's built-in dark theme
- **Preset themes** -- curated color palettes (Crimson, Loki)
- **Custom** -- user picks an accent and background color, all other shades are auto-derived

## Available Themes

| Theme | Author | Accent | Background | Description |
|-------|--------|--------|------------|-------------|
| Default Dark | Root Communications | `#3B6AF8` | `#0D1521` | Root's built-in dark theme (no overrides) |
| Crimson | watchthelight | `#C42B1C` | `#241414` | Deep red accent theme |
| Loki | watchthelight | `#2A5A40` | `#0F1210` | Gold and green theme |
| Custom | (user) | user-defined | user-defined | Generated from two colors |

Theme definitions live in `themes.json`. Each preset specifies 10 CSS variable overrides.

## CSS Variables

The plugin controls these variables:

| Variable | Role |
|----------|------|
| `--rootsdk-brand-primary` | Primary accent color |
| `--rootsdk-brand-secondary` | Lighter accent tint |
| `--rootsdk-brand-tertiary` | Darker accent tint |
| `--rootsdk-background-primary` | Main background |
| `--rootsdk-background-secondary` | Panel/card background |
| `--rootsdk-background-tertiary` | Subtle/recessed background |
| `--rootsdk-input` | Input field background |
| `--rootsdk-border` | Border color |
| `--rootsdk-link` | Link text color |
| `--rootsdk-muted` | Muted/disabled text |

## Custom Color Generation

When the "Custom" theme is selected, the plugin generates all 10 variables from just two inputs using color math:

```
accent     -> brand-primary   (as-is)
accent     -> brand-secondary (lighten 15%)
accent     -> brand-tertiary  (darken 15%)
accent     -> link            (lighten 30%)
background -> bg-primary      (as-is)
background -> bg-secondary    (lighten 8%)
background -> bg-tertiary     (darken 8%)
background -> input           (darken 5%)
background -> border          (lighten 18%)
background -> muted           (lighten 25% if dark, darken 25% if light)
```

Dark vs. light detection uses WCAG relative luminance: `luminance(bg) < 0.3` means dark theme.

### Color math functions

These are exported from the plugin module and reused by the settings panel for live preview:

- **`darken(hex, percent)`** -- multiplies each RGB channel by `(1 - percent/100)`
- **`lighten(hex, percent)`** -- interpolates each RGB channel toward 255 by `percent/100`
- **`generateCustomVariables(accent, bg)`** -- returns a `Record<string, string>` of all 10 variables

## Lifecycle

**start():**
1. Flushes ALL known theme variables (prevents stale values from a previous theme leaking through)
2. Reads theme name from `window.__UPROOTED_SETTINGS__.plugins.themes.config.theme`
3. If "custom": reads `customAccent` and `customBackground` from settings, generates variables, applies them
4. If preset: looks up theme in `themes.json`, applies its `variables` map directly

**stop():**
1. Removes all known CSS variables (from every preset + custom names)

## Settings

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `theme` | select | `"default"` | Which theme to apply. Options are loaded from `themes.json` |

Custom colors (`customAccent`, `customBackground`) are stored in settings but managed by the settings-panel's themes page rather than the theme plugin's own settings schema.

## Known Limitations

- **Web layer only** -- this plugin only themes the Chromium/web layer (WebRTC, sub-apps). Native Avalonia UI (chat, sidebar, settings) is themed by the C# ThemeEngine independently.
- **Session-only** -- theme changes via the settings panel don't persist to disk. The installer writes initial theme settings; runtime changes reset on restart.
- **Variable names are hardcoded** -- if Root adds new `--rootsdk-*` variables, the plugin won't override them
- **No transition animation** -- theme switches are instant CSS variable updates with no transition

---

**Canonical for:** CSS theme plugin behavior, variable overrides, color math, custom theme generation
**Not canonical for:** native Avalonia theme engine → [THEME_ENGINE_DEEP_DIVE.md](../../framework/THEME_ENGINE_DEEP_DIVE.md)
*Themes plugin reference. Last updated 2026-02-19.*
