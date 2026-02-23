# Advanced Plugin Development

> **What this is:** Advanced plugin development patterns — bridge chaining, multi-plugin coordination, performance optimization, error recovery, theme integration.
> **Read when:** Building complex plugins; coordinating between plugins; optimizing performance; integrating with themes.
> **Skip if:** You're writing your first plugin → [GETTING_STARTED.md](GETTING_STARTED.md). You need the API reference → [API_REFERENCE.md](API_REFERENCE.md).

Patterns, best practices, and techniques for experienced Uprooted plugin authors.

> **Related docs:** [Getting Started](GETTING_STARTED.md) | [API Reference](API_REFERENCE.md) | [Bridge Reference](BRIDGE_REFERENCE.md) | [Root Environment](ROOT_ENVIRONMENT.md) | [Examples](EXAMPLES.md)

## Table of Contents

- [Overview](#overview)
- [Bridge Call Chains](#bridge-call-chains)
- [Multi-Plugin Interaction](#multi-plugin-interaction)
- [Performance Best Practices](#performance-best-practices)
- [Error Recovery](#error-recovery)
- [DOM Manipulation Patterns](#dom-manipulation-patterns)
- [Storage Patterns](#storage-patterns)
- [Testing Plugins](#testing-plugins)
- [Theme Integration](#theme-integration)
- [Publishing and Distribution](#publishing-and-distribution)

---

## Overview

This document is for plugin authors who have completed the [Getting Started](GETTING_STARTED.md) tutorials and are comfortable with the basic plugin lifecycle, bridge patching, and DOM injection. It covers advanced patterns for building production-quality plugins -- bridge call chaining, multi-plugin coordination, performance tuning, error recovery, and more.

Prerequisites: you should understand the `UprootedPlugin` interface, the `Patch` system, the CSS and DOM APIs. If any of those are unfamiliar, read [API Reference](API_REFERENCE.md) first.

---

## Bridge Call Chains

The bridge proxy dispatches events to plugin patch handlers in registration order. Multiple plugins can patch the same method, and each handler sees the effects of handlers that ran before it.

### Chaining Multiple Interceptors

When two plugins patch the same bridge method, both `before` handlers run sequentially in registration order. Both execute before the original method is called.

### Modifying Arguments Before Forwarding

The `args` array is mutable. Changes made by an earlier handler are visible to later handlers and to the original method:

```typescript
// Plugin A: clamp noise gate threshold
{
  bridge: "nativeToWebRtc",
  method: "setNoiseGateThreshold",
  before(args) {
    if ((args[0] as number) > 0.05) args[0] = 0.05;
  },
}

// Plugin B (registered later) sees the clamped value in args[0]
```

### Modifying Return Values

The `replace` handler provides a custom return value. Since `replace` cancels the original call, later handlers in the chain do not run:

```typescript
{
  bridge: "webRtcToNative",
  method: "getUserProfile",
  replace(...args) {
    const userId = args[0] as string;
    return { userId, displayName: "Custom Name", avatarUrl: "https://example.com/avatar.png" };
  },
}
```

Be cautious: `replace` blocks all subsequent handlers for the same method.

### Canceling Calls Conditionally

Return `false` from `before` to cancel the original call. This also prevents later handlers from executing:

```typescript
{
  bridge: "nativeToWebRtc",
  method: "setDeafen",
  before(args) {
    // Only allow deafen, block undeafen
    if (!(args[0] as boolean)) return false;
  },
}
```

### Error Injection for Testing

Use `replace` to simulate error conditions during development:

```typescript
{
  bridge: "webRtcToNative",
  method: "failed",
  replace() {
    window.__webRtcToNative?.failed?.({ code: "TEST_ERROR", message: "Simulated failure" } as any);
  },
}
```

---

## Multi-Plugin Interaction

Plugins share a single JavaScript execution context. Understanding coexistence is critical.

### Plugin Load Order and Priority

Plugins start in registration order (see `pluginLoader.ts:96`). Built-in TypeScript plugins register first: `sentry-blocker`, `themes`, `settings-panel`, `link-embeds`, `silent-typing`, then your plugins. This order determines patch handler priority. C# hook plugins (`clear-urls`, `message-logger`, `content-filter`) run independently in the .NET process.

### Shared State Patterns

Expose state on `window` under a namespaced key:

```typescript
// Producer plugin:
start() {
  (window as any).__uprooted_myPlugin = {
    version: "1.0",
    getState() { return { connected: true }; },
  };
}
stop() { delete (window as any).__uprooted_myPlugin; }

// Consumer plugin:
start() {
  const api = (window as any).__uprooted_myPlugin;
  if (api) console.log(api.getState());
}
```

**Anti-patterns:** mutating another plugin's state directly, assuming a dependency is active without checking, using generic names on `window` (always prefix with `__uprooted_`).

### Event Bus for Inter-Plugin Communication

Use `CustomEvent` on `window` for decoupled publish/subscribe:

```typescript
// Publisher:
window.dispatchEvent(new CustomEvent("uprooted:voice-stats:update", { detail: { count: 3 } }));

// Subscriber:
let handler: ((e: Event) => void) | null = null;
start() {
  handler = (e) => console.log((e as CustomEvent).detail);
  window.addEventListener("uprooted:voice-stats:update", handler);
}
stop() {
  if (handler) window.removeEventListener("uprooted:voice-stats:update", handler);
  handler = null;
}
```

Convention: prefix custom event names with `uprooted:` followed by your plugin name.

### Avoiding Conflicts

- **DOM:** Use `uprooted-{pluginName}-{element}` for element IDs and class names.
- **CSS:** Never use generic class names. Scope rules to plugin-specific selectors.
- **Bridge:** If you only observe (not modify), do not mutate `args` and return `undefined` (not `false`).

---

## Performance Best Practices

Plugins run in Root's Chromium process. Poor performance degrades the call experience.

### Avoiding Expensive DOM Queries

Cache element references instead of querying on every bridge event:

```typescript
let indicatorRef: HTMLElement | null = null;
start() { indicatorRef = document.getElementById("my-indicator"); }

// In patch handler:
before(args) { if (indicatorRef) indicatorRef.textContent = String(args[0]); }
```

### Debouncing Bridge Interceptors

High-frequency methods (`setSpeaking`, `receiveRawPacket`) fire many times per second. Debounce expensive work:

```typescript
function debounce<T extends (...a: any[]) => void>(fn: T, ms: number): T {
  let timer: ReturnType<typeof setTimeout> | null = null;
  return ((...a: any[]) => { if (timer) clearTimeout(timer); timer = setTimeout(() => fn(...a), ms); }) as T;
}

const debouncedUpdate = debounce((speaking: boolean) => updateUI(speaking), 100);
// Use debouncedUpdate(args[0]) in your before handler
```

### Memory Leak Prevention

Every resource created in `start()` must be released in `stop()`:

```typescript
let observer: (() => void) | null = null;
let interval: ReturnType<typeof setInterval> | null = null;
let handler: ((e: Event) => void) | null = null;

start() {
  observer = observe(document.body, () => { /* ... */ });
  interval = setInterval(() => { /* ... */ }, 5000);
  handler = () => { /* ... */ };
  window.addEventListener("resize", handler);
}
stop() {
  observer?.(); observer = null;
  if (interval) clearInterval(interval); interval = null;
  if (handler) window.removeEventListener("resize", handler); handler = null;
  document.getElementById("my-plugin-element")?.remove();
}
```

### Lazy Initialization

Defer expensive work until actually needed:

```typescript
let initialized = false;
function ensureInit() {
  if (initialized) return;
  initialized = true;
  // ... expensive setup ...
}

// In a patch handler for "initialize":
before(args) { ensureInit(); }
```

---

## Error Recovery

Root can update at any time, breaking assumptions about DOM, bridge methods, or CSS classes.

### Try/Catch Around Bridge Calls

Patch handlers that throw propagate errors to Root's own code. Always wrap handler logic:

```typescript
before(args) {
  try {
    applyCustomLogic(args[0] as string);
  } catch (err) {
    console.error("[MyPlugin] handler failed:", err);
    // Return undefined to allow the original call to proceed
  }
}
```

For `replace` handlers, return a sensible default on error.

### Graceful Degradation

Design plugins to function with reduced capability when dependencies are missing:

```typescript
async start() {
  try {
    const target = await waitForElement<HTMLElement>("[data-panel='voice']", 5000);
    injectUI(target);
  } catch {
    nativeLog("[MyPlugin] Target not found. Running in degraded mode.");
  }
}
```

### Version Detection and Feature Flags

```typescript
start() {
  const version = window.__UPROOTED_VERSION__ ?? "0.0.0";
  const [major, minor] = version.split(".").map(Number);
  if (major === 0 && minor < 92) {
    nativeLog("[MyPlugin] Requires Uprooted 0.1.92+. Some features disabled.");
    return;
  }
  if (!window.__nativeToWebRtc) {
    nativeLog("[MyPlugin] No active voice session. Deferring.");
    return;
  }
  initFull();
}
```

### Logging Best Practices

- **`nativeLog`**: for startup, shutdown, and errors (persisted to Root's .NET logs).
- **`console.log`**: for development-only messages (visible only with a debugger).
- **High-frequency events**: never log every occurrence. Log state transitions or periodic summaries.

---

## DOM Manipulation Patterns

### Safe Injection Points

Elements appended directly to `document.body` with `position: fixed` and high `z-index` survive React re-renders, since React manages content inside its root container, not `body` itself.

### MutationObserver Patterns

Use the `observe` helper for simple cases. For targeted matching of added nodes:

```typescript
let observer: MutationObserver | null = null;
start() {
  observer = new MutationObserver((mutations) => {
    for (const m of mutations)
      for (const node of m.addedNodes)
        if (node instanceof HTMLElement && node.matches("[data-user-tile]"))
          decorateUserTile(node);
  });
  observer.observe(document.body, { childList: true, subtree: true });
}
stop() { observer?.disconnect(); observer = null; }
```

Keep observer callbacks fast. Use `requestAnimationFrame` or debouncing for expensive work.

### Shadow DOM Considerations

Root does not currently use Shadow DOM in the WebRTC context. If a future update adds shadow roots, `querySelector` from `document` will not reach inside them. Defensive code can check `element.shadowRoot` when walking the tree.

### CSS Scoping

Scope styles using plugin-specific data attributes:

```typescript
css: `
  [data-uprooted-plugin="my-plugin"] .toast {
    background: var(--color-background-secondary, #121A26);
    border-radius: 8px;
  }
`,
start() {
  const container = document.createElement("div");
  container.setAttribute("data-uprooted-plugin", "my-plugin");
  document.body.appendChild(container);
}
```

This prevents style leakage in both directions -- your styles stay contained, and Root's styles do not override yours.

---

## Storage Patterns

Root runs Chromium with `--incognito`, so `localStorage` and `IndexedDB` are unavailable. Plugin settings are the primary persistence mechanism.

### Structured Data Storage

For complex data, serialize to a `string` setting:

```typescript
settings: {
  savedState: { type: "string", default: "{}", description: "Internal state" },
},
start() {
  const raw = (window.__UPROOTED_SETTINGS__?.plugins?.["my-plugin"]?.config?.savedState as string) ?? "{}";
  let state: Record<string, unknown>;
  try { state = JSON.parse(raw); } catch { state = {}; }
}
```

Settings persist only when the user saves through the settings panel. Plugins cannot write to `__UPROOTED_SETTINGS__` and have it persist automatically.

### Migration Between Versions

Include a `schemaVersion` number setting to detect stale config:

```typescript
start() {
  const config = window.__UPROOTED_SETTINGS__?.plugins?.["my-plugin"]?.config;
  const v = (config?.schemaVersion as number) ?? 0;
  if (v < 1) { /* migrate v0 -> v1 */ }
  if (v < 2) { /* migrate v1 -> v2 */ }
}
```

### Shared Storage

For cross-plugin persistence, designate a "data provider" plugin that owns the setting. Other plugins read from the provider's exported `window` object (see [Shared State Patterns](#shared-state-patterns)).

---

## Testing Plugins

Root's Chromium has no DevTools, so testing requires creative approaches.

### Manual Testing Workflow

1. Edit `src/plugins/your-plugin/index.ts`.
2. `pnpm build` to produce `dist/uprooted-preload.js`.
3. `powershell -File Install-Uprooted.ps1` to deploy.
4. Restart Root, join a voice channel.
5. Verify via visual output or `nativeLog` messages.

### Mock Bridge for Unit Testing

Test patch handlers outside of Root with a minimal harness:

```typescript
function mockEvent(method: string, args: unknown[]) {
  return { method, args, cancelled: false, returnValue: undefined };
}
const event = mockEvent("setTheme", ["dark"]);
const result = myPlugin.patches![0].before!(event.args);
console.log(result === false ? "Cancelled" : "Allowed, args:", event.args);
```

### Common Test Scenarios

| Scenario | Trigger | Verify |
|----------|---------|--------|
| Clean start | Enable plugin, restart Root | No error banner, `nativeLog` startup message |
| Clean stop | Disable in settings, restart | No leftover DOM elements |
| Bridge interception | Join voice, toggle mute | `nativeLog` shows intercepted call |
| DOM re-injection | Navigate away and back | Injected element reappears |
| Settings read | Change setting, restart | Behavior reflects new value |
| Error handling | Break a depended-on selector | Graceful degradation, logged error |

---

## Theme Integration

For the complete CSS variable catalog and Root's theme variant system, see [ROOT_ENVIRONMENT.md -- CSS Variables](ROOT_ENVIRONMENT.md#css-variable-system).

### Reading Current Theme

```typescript
import { getCurrentTheme } from "../../api/native.js";
const theme = getCurrentTheme(); // "dark" | "light" | "pure-dark" | null
```

Read specific CSS variable values at runtime with `getComputedStyle(document.documentElement).getPropertyValue("--color-background-primary")`.

### Responding to Theme Changes

Intercept `setTheme` to react when the user switches themes:

```typescript
{ bridge: "nativeToWebRtc", method: "setTheme", before(args) { updatePluginTheme(args[0] as string); } }
```

For CSS-only theme awareness, use Root's CSS variables instead of hardcoded colors:

```css
#my-plugin-panel {
  background: var(--color-background-secondary, #121A26);
  color: var(--color-text-primary, #F2F2F2);
  border: 1px solid var(--color-border, #2A3A4E);
}
```

These variables update automatically when Root's theme changes.

### Theme-Aware UI Elements

Combine CSS variables for default styling with a `setTheme` interceptor for mode-specific adjustments (e.g., different accent for `pure-dark`). Add `transition` properties to your CSS for smooth theme switches. See the [CSS Theme Switcher example](EXAMPLES.md#css-theme-switcher) for a complete implementation.

---

## Publishing and Distribution

### Metadata Requirements

| Field | Purpose | Guidelines |
|-------|---------|------------|
| `name` | Unique identifier | Lowercase, hyphenated. Must not conflict with built-ins (`sentry-blocker`, `themes`, `link-embeds`, `clear-urls`, `message-logger`, `silent-typing`, `content-filter`). |
| `description` | Human-readable summary | One sentence, shown in the settings panel. |
| `version` | Semver string | Follow semantic versioning. Bump on every release. |
| `authors` | Attribution | At least one entry with a `name` field. |

### Plugin File Structure

```
src/plugins/my-plugin/
  index.ts       # Main export (satisfies UprootedPlugin)
  styles.ts      # CSS strings (optional)
  utils.ts       # Helpers (optional)
```

Register in `src/core/preload.ts` with `loader.register(myPlugin)`.

### Packaging

Uprooted bundles all plugins into `dist/uprooted-preload.js` via esbuild. There is no runtime plugin loading. To distribute: fork the public repo, add your plugin, register it, build with `pnpm build`, and distribute the resulting bundle. A dynamic loader is planned for a future release.

### Pre-Distribution Checklist

- [ ] `stop()` cleans up all DOM elements, observers, intervals, and event listeners
- [ ] All patch handlers are wrapped in try/catch
- [ ] CSS uses plugin-specific selectors (no generic class names)
- [ ] Settings have sensible defaults and descriptions
- [ ] `nativeLog` used for startup/error messages (not excessive logging)
- [ ] Plugin degrades gracefully when expected DOM elements are missing
- [ ] No hardcoded colors -- use CSS variables for theme compatibility
- [ ] Element IDs use the `uprooted-{pluginName}-` prefix
- [ ] No use of `localStorage` or `IndexedDB` (unavailable in incognito mode)
- [ ] Tested with a fresh Root install (no stale state)

---

**Canonical for:** advanced plugin patterns, bridge chaining, multi-plugin coordination, performance optimization, error recovery strategies, theme-aware plugin development
**Not canonical for:** basic tutorial → [GETTING_STARTED.md](GETTING_STARTED.md) | API contracts → [API_REFERENCE.md](API_REFERENCE.md) | CSS variable catalog → [ROOT_ENVIRONMENT.md](ROOT_ENVIRONMENT.md)
*Advanced plugin development guide. Last updated 2026-02-19.*
