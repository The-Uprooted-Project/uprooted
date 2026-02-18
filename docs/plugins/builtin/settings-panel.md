# Settings Panel

Injects an "UPROOTED" section into Root's settings sidebar with three pages: Uprooted (about), Plugins (management), and Themes (customization).

> **Source:** [`src/plugins/settings-panel/`](../../../src/plugins/settings-panel/) -- `index.ts`, `panel.ts`, `components.ts`

---

## What it does

When the user opens Root's settings page, this plugin:

1. Detects the settings sidebar by searching for "APP SETTINGS" text in the DOM
2. Clones Root's own sidebar elements to create matching "UPROOTED" navigation items
3. Adds three pages: **Uprooted** (about/links), **Plugins** (toggle list), **Themes** (selector + custom colors + custom CSS)
4. Swaps Root's content panel with Uprooted's pages when nav items are clicked
5. Restores Root's content when the user clicks back to a Root settings item

## Pages

### Uprooted

About page showing version, description, and links to GitHub and the project website. Includes a notice that runtime changes are session-only.

### Plugins

Lists all registered plugins (except settings-panel itself) with:
- **Toggle switch** -- start/stop the plugin at runtime
- **Status badge** -- Active or Inactive
- **Name, version, description** from plugin metadata

The sentry-blocker entry includes a privacy notice explaining what data Root sends to Sentry.

### Themes

- **Theme dropdown** -- select from presets or "Custom"
- **Theme preview cards** -- click-to-select cards showing color swatches for each preset
- **Custom color pickers** -- accent and background `<input type="color">` with live preview (visible only when "Custom" is selected)
- **Custom CSS textarea** -- inject arbitrary CSS with 300ms debounce

## How it works

### DOM Discovery

Root's settings page has no stable selectors -- class names are generated and IDs are absent. The plugin uses text-content matching via `TreeWalker`:

1. Find the leaf element containing exactly `"APP SETTINGS"` -- confirms we're on the settings page
2. Find the leaf element containing exactly `"Advanced"` -- the last item in the settings group
3. Walk up from "APP SETTINGS" to find a flex-row ancestor with 2+ children (the sidebar + content layout)
4. Identify the sidebar child (contains the text) and content child (larger sibling)
5. Walk up from "Advanced" to find the item-level element (the clickable nav item template)
6. Clone the template to create Uprooted nav items with matching styles

### MutationObserver Strategy

Root re-renders the settings page on navigation, destroying injected elements. The plugin:
- Watches `document.body` with `childList: true, subtree: true`
- Debounces at 80ms (clears timer on each mutation, reschedules `tryInject`)
- Checks `injected` flag and verifies `[data-uprooted]` elements still exist in the DOM
- Re-injects if elements were removed (settings page re-rendered)

### Content Panel Swapping

When the user clicks an Uprooted nav item:
1. Deactivate all Root sidebar items (remove active/selected/current classes)
2. Hide Root's content panel (`display: none`)
3. Build the requested page via `buildPage(name)`
4. Append the page as a sibling of Root's content panel

When the user clicks a Root sidebar item:
1. Remove Uprooted content
2. Restore Root's content panel (`display: ""`)
3. Deactivate Uprooted sidebar items

### Element Cloning

Cloned elements go through cleanup:
- Replace text content (only first text node, clear extras like badges)
- Remove active/selected/current CSS classes
- Strip React internal attributes (`__react*`, `data-reactid*`)
- Remove `id` attributes to avoid duplicates
- Remove `href` on anchor tags to prevent navigation

### Version Injection

Searches for "Root Version:" text, then inserts "Uprooted Version: {version}" below it, inheriting Root's font styling.

## UI Components

The plugin includes a small component library in `components.ts`:

| Component | Function | Description |
|-----------|----------|-------------|
| Toggle | `createToggle(checked, onChange)` | Checkbox styled as a slide switch |
| Select | `createSelect(options, selected, onChange)` | Styled dropdown |
| Textarea | `createTextarea(value, placeholder, onChange)` | Monospace textarea with 300ms debounce |
| Row | `createRow(label, description, control)` | Label + description + control layout |
| Section | `createSection(label)` | Uppercase section header |

## Settings

None. The settings-panel itself has no user-configurable options. It renders settings for other plugins.

## Known Limitations

- **Text-based discovery is fragile** -- if Root renames "APP SETTINGS" or "Advanced", sidebar injection silently fails
- **Private Map access** -- `getRegisteredPlugins()` casts `loader` to `any` to access the private `plugins` Map
- **Settings don't persist** -- runtime changes (plugin toggles, theme switches) are session-only
- **Debug overlay always enabled** -- `DEBUG = true` is hardcoded, showing a green-on-black log overlay at the bottom of the page
