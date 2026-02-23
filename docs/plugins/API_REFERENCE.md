# API Reference

> **What this is:** Plugin API surface reference — interface contracts, method signatures, lifecycle hooks, error handling, settings schema.
> **Read when:** Looking up plugin interface details; understanding method signatures; checking settings field types.
> **Skip if:** You need a tutorial → [GETTING_STARTED.md](GETTING_STARTED.md). You need bridge methods → [BRIDGE_REFERENCE.md](BRIDGE_REFERENCE.md).

Complete reference for the Uprooted plugin API. All types are defined in `src/types/plugin.ts` and the API modules under `src/api/`.

> **Related Docs:**
> [Getting Started](GETTING_STARTED.md) |
> [Bridge Reference](BRIDGE_REFERENCE.md) |
> [Root Environment](ROOT_ENVIRONMENT.md) |
> [TypeScript Reference](../framework/TYPESCRIPT_REFERENCE.md)

## Table of Contents

- [UprootedPlugin Interface](#uprootedplugin-interface)
- [Patch Interface](#patch-interface)
- [Settings Definition](#settings-definition)
- [CSS API](#css-api)
- [DOM API](#dom-api)
- [Native API](#native-api)
- [Bridge API](#bridge-api)
- [Global Objects](#global-objects)
- [PluginLoader Class](#pluginloader-class)
- [BridgeEvent Interface](#bridgeevent-interface)
- [Error Handling](#error-handling)
- [Plugin-to-Plugin Communication](#plugin-to-plugin-communication)

---

## UprootedPlugin Interface

Every plugin must export a default object satisfying `UprootedPlugin`. This is the contract between your plugin and the Uprooted loader.

Defined in `src/types/plugin.ts:29`.

```typescript
interface UprootedPlugin {
  name: string;           // Unique identifier (used as key in settings, CSS IDs, etc.)
  description: string;    // Human-readable description shown in the plugins page
  version: string;        // Semver version string
  authors: Author[];      // List of plugin authors

  start?(): void | Promise<void>;   // Called when the plugin is enabled
  stop?(): void | Promise<void>;    // Called when the plugin is disabled

  patches?: Patch[];                // Bridge method intercepts (applied on start, removed on stop)
  css?: string;                     // CSS string injected while the plugin is active
  settings?: SettingsDefinition;    // Plugin-specific settings schema
}

interface Author {
  name: string;
  id?: string;   // Optional unique identifier
}
```

Defined in `src/types/plugin.ts:1` (Author) and `src/types/plugin.ts:29` (UprootedPlugin).

### Required Fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | `string` | Unique plugin identifier. Used as the key in settings JSON, CSS element IDs (`uprooted-css-plugin-{name}`), and log messages. Must be unique across all registered plugins. |
| `description` | `string` | Shown in the Uprooted settings panel plugins page. Keep it short (one sentence). |
| `version` | `string` | Plugin version, displayed alongside the name in the plugins page. |
| `authors` | `Author[]` | At least one author. The `id` field is optional. |

### Optional Fields

| Field | Type | Description |
|-------|------|-------------|
| `start` | `() => void \| Promise<void>` | Lifecycle hook called when the plugin starts. Use this for DOM setup, event listeners, or any initialization. Can be async. |
| `stop` | `() => void \| Promise<void>` | Lifecycle hook called when the plugin stops. Clean up anything your `start` created -- DOM nodes, intervals, event listeners. Can be async. |
| `patches` | `Patch[]` | Array of bridge method intercepts. Automatically installed on start and removed on stop. See [Patch Interface](#patch-interface). |
| `css` | `string` | Raw CSS string. Injected as a `<style>` element on start, removed on stop. Element ID: `uprooted-css-plugin-{name}`. |
| `settings` | `SettingsDefinition` | Defines configurable fields that appear in the settings UI. Values are persisted in the settings JSON. See [Settings Definition](#settings-definition). |

### Lifecycle Order

When a plugin starts (see `src/core/pluginLoader.ts:40`, `start()` method):
1. `patches` are installed (bridge intercepts become active)
2. `css` is injected into the page
3. `start()` is called

When a plugin stops (see `src/core/pluginLoader.ts:73`, `stop()` method):
1. `stop()` is called
2. `css` is removed from the page
3. All event handlers for this plugin's patches are removed

---

## Patch Interface

Patches let you intercept bridge method calls without touching the bridge directly. The loader installs and removes them automatically with the plugin lifecycle.

Defined in `src/types/plugin.ts:6`.

```typescript
interface Patch {
  bridge: "nativeToWebRtc" | "webRtcToNative";
  method: string;
  before?(args: unknown[]): boolean | void | Promise<boolean | void>;
  after?(result: unknown, args: unknown[]): void | Promise<void>;
  replace?(...args: unknown[]): unknown | Promise<unknown>;
}
```

### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `bridge` | `"nativeToWebRtc" \| "webRtcToNative"` | Yes | Which bridge to intercept. See [BRIDGE_REFERENCE.md](BRIDGE_REFERENCE.md). |
| `method` | `string` | Yes | The method name on the bridge to intercept. Must match exactly. |
| `before` | `(args: unknown[]) => boolean \| void` | No | Called before the original method executes. Return `false` to cancel the call. You can mutate `args` in-place to modify what the original receives. |
| `after` | `(result: unknown, args: unknown[]) => void` | No | **Not yet implemented.** Defined in the interface for future use. Currently, the loader only invokes `before` and `replace` handlers. |
| `replace` | `(...args: unknown[]) => unknown` | No | Completely replaces the original method. The original is never called. Your return value is used instead. |

### Execution Priority

- `replace` takes priority over `before`. If `replace` is set, it runs instead of the original method, and `before` is ignored. See the handler logic in `src/core/pluginLoader.ts:126`.
- `before` runs if `replace` is not set. If it returns `false`, the original method is skipped.
- `after` is defined in the interface but **not yet invoked by the loader**. It will be implemented in a future version.
- Multiple plugins can patch the same method. They execute in registration order. If any plugin cancels the call, later plugins' handlers for that method are skipped. This is controlled by the `event.cancelled` check in `src/core/pluginLoader.ts:113`.

### Examples

```typescript
// Log every theme change before it happens
{
  bridge: "nativeToWebRtc",
  method: "setTheme",
  before(args) {
    console.log("Theme changing to:", args[0]);
    // Return nothing (undefined) to allow the call to proceed
  }
}

// Block kick commands
{
  bridge: "nativeToWebRtc",
  method: "kick",
  before(args) {
    console.log("Blocked kick for user:", args[0]);
    return false;  // Cancel the call
  }
}

// Replace disconnect to add custom cleanup
{
  bridge: "nativeToWebRtc",
  method: "disconnect",
  replace() {
    console.log("Custom disconnect handler");
    // Original disconnect is NOT called
  }
}
```

---

## Settings Definition

Define configurable settings for your plugin. These are rendered in the Uprooted settings panel and persisted in the settings JSON file.

Defined in `src/types/plugin.ts:19`.

```typescript
interface SettingsDefinition {
  [key: string]: SettingField;
}

type SettingField =
  | { type: "boolean"; default: boolean; description: string }
  | { type: "string"; default: string; description: string }
  | { type: "number"; default: number; description: string; min?: number; max?: number }
  | { type: "select"; default: string; description: string; options: string[] };
```

Defined in `src/types/plugin.ts:23` (SettingField).

### Field Types

| Type | Rendered As | Value Type | Extra Fields |
|------|------------|------------|--------------|
| `boolean` | Toggle switch | `boolean` | None |
| `string` | Text input | `string` | None |
| `number` | Number input | `number` | `min?`, `max?` |
| `select` | Dropdown | `string` | `options: string[]` |

### Reading Settings at Runtime

Plugin settings are stored in `window.__UPROOTED_SETTINGS__`. Access your plugin's config:

```typescript
const settings = window.__UPROOTED_SETTINGS__?.plugins?.["my-plugin"]?.config;
const myValue = settings?.myKey as string ?? "default-fallback";
```

### Example

```typescript
settings: {
  enabled: {
    type: "boolean",
    default: true,
    description: "Enable this feature"
  },
  username: {
    type: "string",
    default: "",
    description: "Your display name override"
  },
  volume: {
    type: "number",
    default: 50,
    description: "Notification volume",
    min: 0,
    max: 100
  },
  theme: {
    type: "select",
    default: "auto",
    description: "Color scheme",
    options: ["auto", "dark", "light"]
  }
}
```

---

## CSS API

Defined in `src/api/css.ts`. Manages `<style>` elements in the page head.

### `injectCss(id, css)`

Inject a CSS string into the page. If a style with the same ID already exists, its content is replaced.

Defined in `src/api/css.ts:14`.

```typescript
function injectCss(id: string, css: string): void
```

- **id** -- Unique identifier. The actual element ID will be `uprooted-css-{id}`.
- **css** -- Raw CSS string to inject.

The function creates a `<style>` element in `<head>` with the given CSS content. If an element with the same computed ID already exists, its `textContent` is replaced rather than creating a duplicate.

> **Note:** When using the `css` field on your plugin, the loader calls `injectCss("plugin-{name}", css)` automatically (see `src/core/pluginLoader.ts:59`). Use this function directly only if you need dynamic CSS injection in `start()`.

### `removeCss(id)`

Remove a previously injected CSS style element by ID.

Defined in `src/api/css.ts:30`.

```typescript
function removeCss(id: string): void
```

### `removeAllCss()`

Remove all Uprooted-injected CSS from the page (all elements with IDs starting with `uprooted-css-`).

Defined in `src/api/css.ts:39`.

```typescript
function removeAllCss(): void
```

Uses `document.querySelectorAll('style[id^="uprooted-css-"]')` to find all injected styles.

### Example

```typescript
import { injectCss, removeCss } from "../api/css.js";

// Inject some custom styles
injectCss("my-plugin-highlight", `
  .some-element { background: red !important; }
`);

// Later, remove them
removeCss("my-plugin-highlight");
```

---

## DOM API

Defined in `src/api/dom.ts`. Utilities for working with Root's DOM.

### `waitForElement(selector, timeout?)`

Wait for an element matching a CSS selector to appear in the DOM. Uses `MutationObserver` internally.

Defined in `src/api/dom.ts:9`.

```typescript
function waitForElement<T extends Element = Element>(
  selector: string,
  timeout?: number   // default: 10000 (10 seconds)
): Promise<T>
```

- Returns a `Promise` that resolves with the element when found.
- Rejects with an `Error` if the element doesn't appear within the timeout. The error message includes the selector and timeout value (e.g. `waitForElement(".sidebar") timed out after 10000ms`).
- If the element already exists, resolves immediately.
- The observer watches `document.body` with `{ childList: true, subtree: true }`.

```typescript
import { waitForElement } from "../api/dom.js";

const sidebar = await waitForElement<HTMLDivElement>(".sidebar-container");
sidebar.style.border = "2px solid red";
```

### `observe(target, callback, options?)`

Observe a DOM element for mutations. Thin wrapper around `MutationObserver`.

Defined in `src/api/dom.ts:44`.

```typescript
function observe(
  target: Element,
  callback: MutationCallback,
  options?: MutationObserverInit   // default: { childList: true, subtree: true }
): () => void   // Returns a disconnect function
```

- Returns a function that disconnects the observer when called. Store this in your plugin and call it in `stop()`.

```typescript
import { observe } from "../api/dom.js";

let disconnect: (() => void) | null = null;

// In start():
const container = document.querySelector(".chat-messages");
if (container) {
  disconnect = observe(container, (mutations) => {
    for (const mutation of mutations) {
      console.log("DOM changed:", mutation.addedNodes.length, "nodes added");
    }
  });
}

// In stop():
disconnect?.();
disconnect = null;
```

### `nextFrame()`

Wait for the next animation frame. Useful for batching DOM reads after writes.

Defined in `src/api/dom.ts:57`.

```typescript
function nextFrame(): Promise<void>
```

Wraps `requestAnimationFrame` in a `Promise`. Use this when you need to read layout properties (like `getBoundingClientRect()`) after modifying the DOM, to ensure the browser has completed a layout pass.

```typescript
import { nextFrame } from "../api/dom.js";

element.style.width = "100px";
await nextFrame();
const computedWidth = element.getBoundingClientRect().width;
```

---

## Native API

Defined in `src/api/native.ts`. Utilities for interacting with Root's native layer.

### `getCurrentTheme()`

Get the current Root theme from the `data-theme` attribute on `<html>`.

Defined in `src/api/native.ts:11`.

```typescript
function getCurrentTheme(): string | null
```

Returns `"dark"`, `"light"`, `"pure-dark"`, or `null` if not set. Reads `document.documentElement.getAttribute("data-theme")`.

### `setCssVariable(name, value)`

Set a CSS custom property on `:root`. This is how themes work -- overriding `--rootsdk-*` variables.

Defined in `src/api/native.ts:19`.

```typescript
function setCssVariable(name: string, value: string): void
```

Calls `document.documentElement.style.setProperty(name, value)`.

```typescript
setCssVariable("--rootsdk-brand-primary", "#ff0000");
```

### `setCssVariables(vars)`

Set multiple CSS variables at once.

Defined in `src/api/native.ts:33`.

```typescript
function setCssVariables(vars: Record<string, string>): void
```

Iterates over the entries and calls `setProperty` for each. Used heavily by the built-in theme plugin (`src/plugins/themes/index.ts:130`).

```typescript
setCssVariables({
  "--rootsdk-brand-primary": "#ff0000",
  "--rootsdk-background-primary": "#1a1a2e",
});
```

### `removeCssVariable(name)`

Remove a CSS variable override, reverting to the stylesheet default.

Defined in `src/api/native.ts:26`.

```typescript
function removeCssVariable(name: string): void
```

Calls `document.documentElement.style.removeProperty(name)`.

### `nativeLog(message)`

Send a log message through Root's native bridge. The message appears in .NET/Root logs, prefixed with `[Uprooted]`.

Defined in `src/api/native.ts:42`.

```typescript
function nativeLog(message: string): void
```

This calls `window.__webRtcToNative?.log()` under the hood, which maps to the `IWebRtcToNative.log()` bridge method (see [Bridge Reference -- log](BRIDGE_REFERENCE.md#log)). It's the only way to get log output visible outside the Chromium context (since there's no DevTools).

```typescript
nativeLog("Plugin initialized successfully");
// Appears in Root logs as: [Uprooted] Plugin initialized successfully
```

---

## Bridge API

Defined in `src/api/bridge.ts`. You generally **do not call bridge methods directly**. Instead, use [Patch](#patch-interface) definitions on your plugin to intercept bridge traffic.

### How It Works

Uprooted replaces Root's two bridge globals (`window.__nativeToWebRtc` and `window.__webRtcToNative`) with ES6 Proxy wrappers. When any code calls a bridge method:

1. The Proxy intercepts the call (via the `get` trap at `src/api/bridge.ts:26`)
2. A `BridgeEvent` is created with the method name and arguments
3. The event is emitted to all registered plugin patch handlers (via `pluginLoader.emit()` at `src/api/bridge.ts:37`)
4. If no handler cancels the event, the original method is called
5. The return value is passed back to the caller

This is transparent to Root's own code -- it doesn't know the bridges are proxied.

### Bridge Installation

The proxies are installed at startup via `installBridgeProxy()` (defined at `src/api/bridge.ts:49`). This handles both cases:
- **Immediate:** If the bridge objects already exist on `window`, they're wrapped right away (line 54-70).
- **Deferred:** If Root assigns them later, `Object.defineProperty` setters intercept the assignment and wrap the value automatically (line 80-102).

### Direct Access

If you need the raw bridge objects (rare), they're available on `window`:

```typescript
// These are the PROXIED versions — your patches will still fire
window.__nativeToWebRtc?.setTheme("dark");
window.__webRtcToNative?.log("hello");
```

There is no way to access the un-proxied originals from plugin code.

---

## Global Objects

These globals are available on `window` inside Root's Chromium context.

### Uprooted Globals

| Global | Type | Description |
|--------|------|-------------|
| `window.__UPROOTED_SETTINGS__` | `UprootedSettings` | The settings object loaded at startup. Contains `enabled`, `plugins` (per-plugin config), and `customCss`. |
| `window.__UPROOTED_VERSION__` | `string` | Uprooted version string (e.g. `"0.2.2"`). Set during initialization. |
| `window.__UPROOTED_LOADER__` | `PluginLoader` | The active plugin loader instance. Exposed for the settings panel; avoid depending on this in regular plugins. |

### Root Bridge Globals

| Global | Type | Description |
|--------|------|-------------|
| `window.__nativeToWebRtc` | `INativeToWebRtc` | Native-to-WebRTC bridge (proxied by Uprooted). C# host calls these to control WebRTC. |
| `window.__webRtcToNative` | `IWebRtcToNative` | WebRTC-to-native bridge (proxied by Uprooted). JS calls these to notify the C# host. |

### Settings Structure

Defined in `src/types/settings.ts`.

```typescript
interface UprootedSettings {
  enabled: boolean;                              // Global kill switch
  plugins: Record<string, PluginSettings>;       // Per-plugin settings
  customCss: string;                             // Global custom CSS
}

interface PluginSettings {
  enabled: boolean;                              // Whether this plugin should start
  config: Record<string, unknown>;               // Plugin-specific config values
}
```

---

## PluginLoader Class

Defined in `src/core/pluginLoader.ts:20`. Manages plugin registration, lifecycle, and bridge event dispatch.

### Methods

#### `register(plugin)`

Register a plugin. Does not start it. If a plugin with the same name is already registered, the call is ignored with a warning.

Defined in `src/core/pluginLoader.ts:31`.

```typescript
register(plugin: UprootedPlugin): void
```

#### `start(name)`

Start a plugin by name. Installs patches, injects CSS, and calls `start()`. No-op if already active.

Defined in `src/core/pluginLoader.ts:40`.

```typescript
start(name: string): Promise<void>
```

Errors thrown during `start()` are caught and logged to `console.error`. The plugin is not marked as active if an error occurs. See [Error Handling](#error-handling) for details.

#### `stop(name)`

Stop a plugin by name. Calls `stop()`, removes CSS, and uninstalls all patch handlers. No-op if not active.

Defined in `src/core/pluginLoader.ts:73`.

```typescript
stop(name: string): Promise<void>
```

#### `startAll()`

Start all registered plugins that are enabled in settings. Plugins default to enabled if not explicitly configured.

Defined in `src/core/pluginLoader.ts:96`.

```typescript
startAll(): Promise<void>
```

Iterates over all registered plugins in registration order. For each plugin, checks `settings.plugins[name]?.enabled ?? true` -- if no explicit setting exists, the plugin is enabled by default.

#### `emit(eventName, event)`

Emit a bridge event. Called by the bridge proxy -- you don't call this directly.

Defined in `src/core/pluginLoader.ts:107`.

```typescript
emit(eventName: "bridge:nativeToWebRtc" | "bridge:webRtcToNative", event: BridgeEvent): void
```

The emit system uses a composite key of `${eventName}:${method}` (e.g. `"bridge:nativeToWebRtc:setTheme"`) to dispatch events only to handlers registered for that specific method. Handlers execute in registration order, and if any handler sets `event.cancelled = true`, subsequent handlers are skipped.

---

## BridgeEvent Interface

The event object passed to patch handlers via the loader's event system.

Defined in `src/core/pluginLoader.ts:11`.

```typescript
interface BridgeEvent {
  method: string;        // The bridge method that was called
  args: unknown[];       // The arguments passed to the method
  cancelled: boolean;    // Set to true to prevent the original call
  returnValue?: unknown; // Set by replace handlers; used as the return value if cancelled
}
```

- `method` and `args` are set by the bridge proxy.
- `cancelled` starts as `false`. Set it to `true` (or return `false` from `before`) to prevent the original method from executing.
- `returnValue` is only used when `cancelled` is `true`. Set it in a `replace` handler to provide a custom return value.

---

## Error Handling

Understanding how errors propagate is important for writing robust plugins.

### Plugin Startup Errors

When `PluginLoader.start()` is called (see `src/core/pluginLoader.ts:40`), the entire startup sequence (patch installation, CSS injection, and `start()` call) is wrapped in a try/catch. If your `start()` function throws:

1. The error is logged to `console.error` with the message `[Uprooted] Failed to start plugin "{name}":`
2. The plugin is **not** added to the active set, meaning it will not receive bridge events
3. Other plugins are unaffected -- the error does not propagate to `startAll()`

```typescript
// This is safe -- the error is caught by the loader
start() {
  throw new Error("Something went wrong");
  // Result: logged to console, plugin stays inactive
}
```

### Plugin Stop Errors

Similarly, `PluginLoader.stop()` (see `src/core/pluginLoader.ts:73`) wraps the shutdown in try/catch. If `stop()` throws:

1. The error is logged to `console.error`
2. CSS removal and handler cleanup still proceed (they run before `stop()` is called... actually `stop()` is called first, then cleanup follows)
3. The plugin is removed from the active set regardless of the error

### Uncaught Errors in Patch Handlers

Patch handlers (`before` and `replace`) are called synchronously by the event emitter (`src/core/pluginLoader.ts:107`). If a patch handler throws, the error will propagate up through the bridge proxy to the original caller (Root's code). This can cause unexpected behavior in Root.

**Best practice:** Always wrap patch handler logic in try/catch:

```typescript
patches: [
  {
    bridge: "nativeToWebRtc",
    method: "setTheme",
    before(args) {
      try {
        // Your logic here
      } catch (err) {
        console.error("[MyPlugin] Error in setTheme handler:", err);
        // Return undefined to allow the original call to proceed
      }
    },
  },
],
```

### Fatal Initialization Errors

If an error occurs during Uprooted's own initialization (in `main()` inside `src/core/preload.ts`), a red banner is displayed at the top of the page showing the error. This covers cases like the bridge proxy failing to install.

### Error Patterns to Avoid

| Pattern | Problem | Solution |
|---------|---------|----------|
| Unguarded `await` in `start()` | `waitForElement` rejection kills the plugin | Wrap in try/catch, log with `nativeLog` |
| Throwing in `replace` handler | Error propagates to Root's own code | Wrap in try/catch, return a sensible default |
| Accessing `window.__nativeToWebRtc` without null check | Throws if no voice session is active | Check for existence: `if (window.__nativeToWebRtc)` |
| Relying on `after` handler | Not yet implemented by the loader | Use `before` for observation, or combine `before` + `replace` |

---

## Plugin-to-Plugin Communication

Plugins share a single JS execution context, which enables several communication patterns. These are not built into the Uprooted API -- they are standard JavaScript patterns that work because all plugins run in the same `window`.

### Shared Window Properties

The simplest approach: put data on `window` under a namespaced key.

```typescript
// Plugin A (producer):
start() {
  (window as any).__myPlugin_state = {
    count: 0,
    increment() { this.count++; },
  };
}

// Plugin B (consumer):
start() {
  const state = (window as any).__myPlugin_state;
  if (state) {
    state.increment();
    console.log("Count:", state.count);
  }
}
```

**Caveats:** Plugin registration order matters. If Plugin B starts before Plugin A, the shared state won't exist yet. Use defensive checks.

### Custom Events on window

Use `CustomEvent` for decoupled publish/subscribe between plugins:

```typescript
// Plugin A (publisher):
start() {
  window.dispatchEvent(new CustomEvent("uprooted:my-plugin:ready", {
    detail: { version: "1.0" },
  }));
}

// Plugin B (subscriber):
start() {
  window.addEventListener("uprooted:my-plugin:ready", ((e: CustomEvent) => {
    console.log("Plugin A is ready:", e.detail);
  }) as EventListener);
}
```

**Convention:** Prefix custom event names with `uprooted:` to avoid collisions with Root's own events.

### Bridge Interception Chaining

Multiple plugins can patch the same bridge method. They execute in registration order. A later plugin can observe what an earlier plugin did by checking the `args` array (which earlier plugins may have mutated):

```typescript
// Plugin A (registered first):
patches: [
  {
    bridge: "nativeToWebRtc",
    method: "setTheme",
    before(args) {
      // Force dark theme
      args[0] = "dark";
    },
  },
],

// Plugin B (registered second):
patches: [
  {
    bridge: "nativeToWebRtc",
    method: "setTheme",
    before(args) {
      // args[0] is now "dark" because Plugin A mutated it
      console.log("Theme after all modifications:", args[0]);
    },
  },
],
```

If Plugin A returns `false` (cancelling the call), Plugin B's handler **will not run** -- the event loop breaks on cancellation (see `src/core/pluginLoader.ts:113`).

### Accessing the Loader

The `PluginLoader` instance is available at `window.__UPROOTED_LOADER__`. While primarily intended for the settings panel plugin, other plugins can use it to query the state of other plugins:

```typescript
const loader = window.__UPROOTED_LOADER__ as any;
const activePlugins: Set<string> = loader.activePlugins;
const isThemesActive = activePlugins.has("themes");
```

**Warning:** This accesses private properties of the loader. The internal API is not stable and may change between versions. Use defensively.

---

**Canonical for:** plugin interface contracts (`UprootedPlugin`, `Patch`, `SettingField`, `Author`), method signatures, lifecycle hooks, error handling, settings schema
**Not canonical for:** tutorial → [GETTING_STARTED.md](GETTING_STARTED.md) | bridge methods → [BRIDGE_REFERENCE.md](BRIDGE_REFERENCE.md) | Root environment → [ROOT_ENVIRONMENT.md](ROOT_ENVIRONMENT.md)
*Plugin API reference. Last updated 2026-02-19.*
