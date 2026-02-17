# api reference

types in `src/types/plugin.ts`, api modules in `src/api/`.

## UprootedPlugin

every plugin exports a default object satisfying this interface:

```typescript
interface UprootedPlugin {
  name: string;
  description: string;
  version: string;
  authors: Author[];
  start?(): void | Promise<void>;
  stop?(): void | Promise<void>;
  patches?: Patch[];
  css?: string;
  settings?: SettingsDefinition;
}

interface Author {
  name: string;
  id?: string;
}
```

`name` is the key for everything: settings json, css element ids (`uprooted-css-plugin-{name}`), logs. must be unique.

lifecycle order on start: patches installed -> css injected -> `start()` called. on stop: `stop()` called -> css removed -> patches removed.

## Patch

intercept bridge method calls. loader installs/removes them with the plugin lifecycle.

```typescript
interface Patch {
  bridge: "nativeToWebRtc" | "webRtcToNative";
  method: string;
  before?(args: unknown[]): boolean | void | Promise<boolean | void>;
  after?(result: unknown, args: unknown[]): void | Promise<void>;
  replace?(...args: unknown[]): unknown | Promise<unknown>;
}
```

- `bridge`: which bridge to intercept
- `method`: exact method name
- `before`: runs before the original. return `false` to cancel. you can mutate `args` in-place
- `after`: not yet implemented, reserved for future use
- `replace`: completely replaces the original method. original never called. takes priority over `before`

multiple plugins can patch the same method. they run in registration order. if any cancels, later handlers are skipped.

```typescript
// log theme changes
{ bridge: "nativeToWebRtc", method: "setTheme", before(args) { console.log("Theme:", args[0]); } }

// block kicks
{ bridge: "nativeToWebRtc", method: "kick", before(args) { return false; } }

// replace disconnect
{ bridge: "nativeToWebRtc", method: "disconnect", replace() { console.log("custom disconnect"); } }
```

## Settings

define configurable fields that show in the settings panel:

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

read at runtime:

```typescript
const settings = window.__UPROOTED_SETTINGS__?.plugins?.["my-plugin"]?.config;
const myValue = settings?.myKey as string ?? "default-fallback";
```

```typescript
settings: {
  enabled: { type: "boolean", default: true, description: "Enable this feature" },
  username: { type: "string", default: "", description: "Your display name override" },
  volume: { type: "number", default: 50, description: "Notification volume", min: 0, max: 100 },
  theme: { type: "select", default: "auto", description: "Color scheme", options: ["auto", "dark", "light"] }
}
```

## CSS API

manages `<style>` elements in the page head.

`injectCss(id, css)` - inject css. element id becomes `uprooted-css-{id}`. replaces content if id already exists.

`removeCss(id)` - remove a previously injected style element.

`removeAllCss()` - remove all uprooted-injected css (ids starting with `uprooted-css-`).

```typescript
import { injectCss, removeCss } from "../api/css.js";

injectCss("my-plugin-highlight", `.some-element { background: red !important; }`);
removeCss("my-plugin-highlight");
```

when using the `css` field on your plugin, the loader calls `injectCss("plugin-{name}", css)` automatically.

## DOM API

`waitForElement<T>(selector, timeout?)` - returns a promise that resolves when an element matching the selector appears. uses MutationObserver internally. default timeout 10s. rejects on timeout.

```typescript
import { waitForElement } from "../api/dom.js";
const sidebar = await waitForElement<HTMLDivElement>(".sidebar-container");
```

`observe(target, callback, options?)` - thin MutationObserver wrapper. returns a disconnect function. default options: `{ childList: true, subtree: true }`.

```typescript
import { observe } from "../api/dom.js";
let disconnect = observe(container, (mutations) => {
  console.log("DOM changed:", mutations.length);
});
// cleanup:
disconnect();
```

`nextFrame()` - promise wrapper around requestAnimationFrame. use after DOM writes before reading layout.

```typescript
import { nextFrame } from "../api/dom.js";
element.style.width = "100px";
await nextFrame();
const width = element.getBoundingClientRect().width;
```

## Native API

`getCurrentTheme()` - returns `"dark"`, `"light"`, `"pure-dark"`, or `null`. reads `data-theme` from `<html>`.

`setCssVariable(name, value)` - set a css custom property on `:root`.

`setCssVariables(vars)` - set multiple at once.

`removeCssVariable(name)` - remove an override, revert to stylesheet default.

`nativeLog(message)` - send a log message through the native bridge. shows up in .NET logs with `[Uprooted]` prefix. only way to get log output outside chromium since there's no devtools.

```typescript
setCssVariable("--rootsdk-brand-primary", "#ff0000");
nativeLog("Plugin initialized");
```

## Bridge API

you don't call bridge methods directly. use Patch definitions to intercept traffic.

uprooted replaces the two bridge globals (`window.__nativeToWebRtc` and `window.__webRtcToNative`) with Proxy wrappers. when any code calls a bridge method, the proxy intercepts it, emits to registered patch handlers, and calls the original if not cancelled. transparent to the app's own code.

```typescript
// direct access (rare, patches still fire on these):
window.__nativeToWebRtc?.setTheme("dark");
window.__webRtcToNative?.log("hello");
```

no way to access un-proxied originals from plugin code.

## Globals

available on `window`:

- `window.__UPROOTED_SETTINGS__` - settings object (enabled, plugins, customCss)
- `window.__UPROOTED_VERSION__` - version string
- `window.__UPROOTED_LOADER__` - the plugin loader instance (mainly for settings panel, avoid depending on it)
- `window.__nativeToWebRtc` - native-to-WebRTC bridge (proxied)
- `window.__webRtcToNative` - WebRTC-to-native bridge (proxied)

settings structure:

```typescript
interface UprootedSettings {
  enabled: boolean;
  plugins: Record<string, PluginSettings>;
  customCss: string;
}

interface PluginSettings {
  enabled: boolean;
  config: Record<string, unknown>;
}
```

## PluginLoader

manages registration, lifecycle, and bridge event dispatch.

- `register(plugin)` - register a plugin. doesn't start it
- `start(name)` - start by name. installs patches, injects css, calls start(). errors are caught and logged
- `stop(name)` - stop by name. calls stop(), removes css, removes patches
- `startAll()` - start all registered plugins that are enabled in settings. defaults to enabled if not configured
- `emit(eventName, event)` - emit a bridge event. called by the proxy, you don't call this

## BridgeEvent

```typescript
interface BridgeEvent {
  method: string;
  args: unknown[];
  cancelled: boolean;
  returnValue?: unknown;
}
```

`cancelled` starts false. set true (or return false from `before`) to prevent the original call. `returnValue` is used when cancelled.

## Error Handling

- plugin start() errors are caught by the loader, logged, plugin stays inactive. other plugins unaffected
- plugin stop() errors are caught, plugin removed from active set regardless
- patch handler errors propagate to the caller. wrap your handlers in try/catch
- fatal init errors show a red banner at the top of the page

## Plugin Communication

plugins share a single JS context. common patterns:

shared window properties:

```typescript
// producer:
(window as any).__myPlugin_state = { count: 0 };
// consumer:
const state = (window as any).__myPlugin_state;
```

custom events:

```typescript
window.dispatchEvent(new CustomEvent("uprooted:my-plugin:ready", { detail: { version: "1.0" } }));
window.addEventListener("uprooted:my-plugin:ready", (e: CustomEvent) => { console.log(e.detail); });
```

prefix custom events with `uprooted:` to avoid collisions. registration order matters for shared state.
