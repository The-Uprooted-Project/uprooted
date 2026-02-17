# getting started

build plugins for uprooted. assumes typescript + DOM knowledge.

## prereqs

- node.js v18+
- pnpm v8+ (`npm install -g pnpm`)
- uprooted installed

## setup

```bash
git clone https://github.com/watchthelight/uprooted.git
cd uprooted
pnpm install
pnpm build
```

should produce `dist/uprooted-preload.js` and `dist/uprooted.css`.

## project structure

```
src/
├── types/
│   ├── plugin.ts      # UprootedPlugin, Patch, SettingsDefinition
│   ├── bridge.ts      # INativeToWebRtc, IWebRtcToNative
│   └── settings.ts    # UprootedSettings
├── api/
│   ├── css.ts         # injectCss, removeCss, removeAllCss
│   ├── dom.ts         # waitForElement, observe, nextFrame
│   ├── native.ts      # getCurrentTheme, setCssVariable(s), nativeLog
│   └── bridge.ts      # bridge proxy internals
├── core/
│   ├── preload.ts     # entry point
│   └── pluginLoader.ts # plugin lifecycle
└── plugins/
    ├── themes/
    ├── sentry-blocker/
    └── settings-panel/
```

## hello world

create `src/plugins/hello-world/index.ts`:

```typescript
import type { UprootedPlugin } from "../../types/plugin.js";
import { nativeLog } from "../../api/native.js";

export default {
  name: "hello-world",
  description: "My first Uprooted plugin",
  version: "0.1.0",
  authors: [{ name: "YourName" }],

  start() {
    nativeLog("Hello World plugin started!");
    const badge = document.createElement("div");
    badge.id = "hello-world-badge";
    badge.textContent = "Hello from Uprooted!";
    badge.style.cssText =
      "position: fixed; bottom: 12px; right: 12px; z-index: 999999; " +
      "padding: 8px 16px; background: #2D7D46; color: #fff; " +
      "font: 14px sans-serif; border-radius: 8px; pointer-events: none;";
    document.body.appendChild(badge);
  },

  stop() {
    document.getElementById("hello-world-badge")?.remove();
    nativeLog("Hello World plugin stopped!");
  },
} satisfies UprootedPlugin;
```

register it in `src/core/preload.ts`:

```typescript
import helloWorldPlugin from "../plugins/hello-world/index.js";
loader.register(helloWorldPlugin);
```

build with `pnpm build`, reinstall, restart the app. green badge should show up bottom-right.

`UprootedPlugin` requires `name`, `description`, `version`, `authors`. `start()` and `stop()` are optional.

## css injection

### static css

declare css on your plugin object, loader handles inject/remove:

```typescript
import type { UprootedPlugin } from "../../types/plugin.js";

export default {
  name: "round-avatars",
  description: "Makes all avatars perfectly round",
  version: "0.1.0",
  authors: [{ name: "YourName" }],

  css: `
    img[class*="avatar"] {
      border-radius: 50% !important;
    }
  `,
} satisfies UprootedPlugin;
```

style element gets id `uprooted-css-plugin-round-avatars`.

### dynamic css

for runtime changes, use the css api directly:

```typescript
import type { UprootedPlugin } from "../../types/plugin.js";
import { injectCss, removeCss } from "../../api/css.js";

export default {
  name: "dynamic-styles",
  description: "Changes styles based on time of day",
  version: "0.1.0",
  authors: [{ name: "YourName" }],

  start() {
    const hour = new Date().getHours();
    const isNight = hour < 6 || hour > 20;
    injectCss("dynamic-styles-time", `
      :root {
        --rootsdk-brand-primary: ${isNight ? "#6366f1" : "#f59e0b"} !important;
      }
    `);
  },

  stop() {
    removeCss("dynamic-styles-time");
  },
} satisfies UprootedPlugin;
```

### css variable overrides

override `--rootsdk-*` variables for theming:

```typescript
import { setCssVariables, removeCssVariable } from "../../api/native.js";

// in start():
setCssVariables({
  "--rootsdk-brand-primary": "#e11d48",
  "--rootsdk-background-primary": "#1a1a2e",
});

// in stop():
removeCssVariable("--rootsdk-brand-primary");
removeCssVariable("--rootsdk-background-primary");
```

## bridge interception

intercept calls between the native host and WebRTC layer.

### before handler

```typescript
import type { UprootedPlugin } from "../../types/plugin.js";

export default {
  name: "theme-logger",
  description: "Logs theme changes",
  version: "0.1.0",
  authors: [{ name: "YourName" }],

  patches: [
    {
      bridge: "nativeToWebRtc",
      method: "setTheme",
      before(args) {
        console.log("Theme changing to:", args[0]);
      },
    },
  ],
} satisfies UprootedPlugin;
```

return `false` from `before` to cancel the call:

```typescript
patches: [
  {
    bridge: "nativeToWebRtc",
    method: "kick",
    before(args) {
      console.log("Blocked kick for:", args[0]);
      return false;
    },
  },
],
```

### replace handler

completely replaces the original method:

```typescript
patches: [
  {
    bridge: "nativeToWebRtc",
    method: "disconnect",
    replace() {
      console.log("Custom disconnect - adding cleanup");
    },
  },
],
```

see [API Reference - Patch Interface](API_REFERENCE.md#patch-interface) for details.

## plugin settings

```typescript
import type { UprootedPlugin } from "../../types/plugin.js";
import { nativeLog } from "../../api/native.js";

export default {
  name: "my-configurable-plugin",
  description: "A plugin with settings",
  version: "0.1.0",
  authors: [{ name: "YourName" }],

  settings: {
    greeting: {
      type: "string",
      default: "Hello!",
      description: "Message shown on startup",
    },
    showBadge: {
      type: "boolean",
      default: true,
      description: "Show a visible badge in the UI",
    },
    badgeSize: {
      type: "number",
      default: 14,
      description: "Badge font size in pixels",
      min: 8,
      max: 32,
    },
    position: {
      type: "select",
      default: "bottom-right",
      description: "Badge position on screen",
      options: ["top-left", "top-right", "bottom-left", "bottom-right"],
    },
  },

  start() {
    const config = window.__UPROOTED_SETTINGS__?.plugins?.["my-configurable-plugin"]?.config;
    const greeting = (config?.greeting as string) ?? "Hello!";
    const showBadge = (config?.showBadge as boolean) ?? true;
    const badgeSize = (config?.badgeSize as number) ?? 14;
    const position = (config?.position as string) ?? "bottom-right";

    nativeLog(greeting);

    if (showBadge) {
      const badge = document.createElement("div");
      badge.id = "my-plugin-badge";
      badge.textContent = greeting;
      const [vertical, horizontal] = position.split("-");
      badge.style.cssText =
        `position: fixed; ${vertical}: 12px; ${horizontal}: 12px; z-index: 999999; ` +
        `padding: 8px 16px; background: #2D7D46; color: #fff; ` +
        `font: ${badgeSize}px sans-serif; border-radius: 8px;`;
      document.body.appendChild(badge);
    }
  },

  stop() {
    document.getElementById("my-plugin-badge")?.remove();
  },
} satisfies UprootedPlugin;
```

settings are stored in `window.__UPROOTED_SETTINGS__?.plugins?.[name]?.config`. always use fallback defaults. see [API Reference - Settings](API_REFERENCE.md#settings-definition).

## dom injection

two problems: elements might not exist yet (async load), and re-renders can destroy your injected content.

use `waitForElement` to wait, `observe` to re-inject:

```typescript
import type { UprootedPlugin } from "../../types/plugin.js";
import { waitForElement, observe } from "../../api/dom.js";
import { nativeLog } from "../../api/native.js";

let disconnect: (() => void) | null = null;

function createStatusBar(): HTMLDivElement {
  const bar = document.createElement("div");
  bar.id = "status-bar-plugin";
  bar.style.cssText =
    "position: fixed; top: 0; left: 0; right: 0; z-index: 999999; " +
    "height: 24px; display: flex; align-items: center; justify-content: center; " +
    "background: #2D7D46; color: #fff; font: 11px monospace; pointer-events: none;";
  bar.textContent = `Uprooted v${window.__UPROOTED_VERSION__ ?? "dev"} | ${new Date().toLocaleTimeString()}`;
  return bar;
}

export default {
  name: "status-bar",
  description: "Injects a persistent status bar at the top of the page",
  version: "0.1.0",
  authors: [{ name: "YourName" }],

  async start() {
    try {
      await waitForElement("body > *", 5000);
      const inject = () => {
        if (document.getElementById("status-bar-plugin")) return;
        document.body.prepend(createStatusBar());
      };
      inject();
      disconnect = observe(document.body, () => {
        if (!document.getElementById("status-bar-plugin")) {
          nativeLog("Status bar removed, re-injecting");
          inject();
        }
      });
    } catch (err) {
      nativeLog(`Status bar plugin failed: ${err}`);
    }
  },

  stop() {
    disconnect?.();
    disconnect = null;
    document.getElementById("status-bar-plugin")?.remove();
  },
} satisfies UprootedPlugin;
```

## build and test

`pnpm build` bundles to `dist/uprooted-preload.js`. `powershell -File Install-Uprooted.ps1` deploys it. launch the app, join a voice channel, check settings for the UPROOTED section.

no devtools available. use `nativeLog()` for logging and DOM elements for visual debugging.

## next

- [API_REFERENCE.md](API_REFERENCE.md) - full api docs
- [EXAMPLES.md](EXAMPLES.md) - copy-paste plugins
- built-in plugins in `src/plugins/` are good references
