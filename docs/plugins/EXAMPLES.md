# Example Plugins

> **What this is:** 11 working code examples — copy-paste plugin starters covering common patterns from basic to advanced.
> **Read when:** Looking for a starting template; seeing concrete code for a specific pattern; learning by example.
> **Skip if:** You need API docs → [API_REFERENCE.md](API_REFERENCE.md). You need step-by-step tutorial → [GETTING_STARTED.md](GETTING_STARTED.md).

Copy-paste example plugins demonstrating common patterns. Each example is a complete, self-contained plugin.

> **Related Docs:**
> [Getting Started](GETTING_STARTED.md) |
> [API Reference](API_REFERENCE.md) |
> [Bridge Reference](BRIDGE_REFERENCE.md)

## Table of Contents

- [Minimal Template](#minimal-template) -- Beginner
- [Theme Logger](#theme-logger) -- Beginner
- [Bridge Event Logger](#bridge-event-logger) -- Beginner
- [Anti-Kick](#anti-kick) -- Beginner
- [Voice Activity Monitor](#voice-activity-monitor) -- Intermediate
- [Custom Theme](#custom-theme) -- Beginner
- [Settings Example](#settings-example) -- Intermediate
- [DOM Injector](#dom-injector) -- Intermediate
- [Notification Interceptor](#notification-interceptor) -- Intermediate
- [Call Logger](#call-logger) -- Advanced
- [CSS Theme Switcher](#css-theme-switcher) -- Advanced

---

## Minimal Template

**Difficulty: Beginner**

The absolute bare minimum plugin. Use this as your starting point.

```typescript
import type { UprootedPlugin } from "../../types/plugin.js";

export default {
  name: "my-plugin",
  description: "Does something cool",
  version: "0.1.0",
  authors: [{ name: "YourName" }],

  start() {
    console.log("[my-plugin] Started");
  },

  stop() {
    console.log("[my-plugin] Stopped");
  },
} satisfies UprootedPlugin;
```

To register: add `import myPlugin from "../plugins/my-plugin/index.js";` and `loader.register(myPlugin);` in `src/core/preload.ts`.

---

## Theme Logger

**Difficulty: Beginner**

Uses a `before` patch handler to log every theme change. Demonstrates basic bridge interception.

```typescript
import type { UprootedPlugin } from "../../types/plugin.js";
import type { Theme } from "../../types/bridge.js";
import { nativeLog } from "../../api/native.js";

export default {
  name: "theme-logger",
  description: "Logs theme changes to native log",
  version: "0.1.0",
  authors: [{ name: "YourName" }],

  patches: [
    {
      bridge: "nativeToWebRtc",
      method: "setTheme",
      before(args) {
        const theme = args[0] as Theme;
        nativeLog(`Theme changed to: ${theme}`);
      },
    },
  ],

  start() {
    const current = document.documentElement.getAttribute("data-theme");
    nativeLog(`Theme Logger active. Current theme: ${current ?? "unknown"}`);
  },
} satisfies UprootedPlugin;
```

**Key concepts:** `before` handler, accessing `args`, using `nativeLog`. See [API Reference -- Patch Interface](API_REFERENCE.md#patch-interface) for handler details.

---

## Bridge Event Logger

**Difficulty: Beginner**

Logs all bridge events to the console for debugging and development. Useful when you want to understand what bridge calls Root makes in response to user actions.

```typescript
import type { UprootedPlugin } from "../../types/plugin.js";
import type { Patch } from "../../types/plugin.js";
import { nativeLog } from "../../api/native.js";

// All known nativeToWebRtc methods (42 methods)
// See BRIDGE_REFERENCE.md for the full list
const nativeToWebRtcMethods = [
  "initialize", "disconnect",
  "setIsVideoOn", "setIsScreenShareOn", "setIsAudioOn",
  "updateVideoDeviceId", "updateAudioInputDeviceId", "updateAudioOutputDeviceId",
  "updateScreenShareDeviceId", "updateScreenAudioDeviceId",
  "updateProfile", "updateMyPermission",
  "setPushToTalkMode", "setPushToTalk",
  "setMute", "setDeafen", "setHandRaised",
  "setTheme", "setNoiseGateThreshold", "setDenoisePower",
  "setScreenQualityMode", "toggleFullFocus",
  "setPreferredCodecs", "setUserMediaConstraints", "setDisplayMediaConstraints",
  "setScreenContentHint", "screenPickerDismissed",
  "setAdminMute", "setAdminDeafen", "kick",
  "setTileVolume", "setOutputVolume", "setInputVolume", "customizeVolumeBooster",
  "receiveRawPacket", "receiveRawPacketContainer", "receivePacket",
  "nativeLoopbackAudioStarted", "receiveNativeLoopbackAudioData",
  "getNativeLoopbackAudioTrack", "stopNativeLoopbackAudio",
];

// All known webRtcToNative methods (29 methods)
const webRtcToNativeMethods = [
  "initialized", "disconnected", "failed",
  "localAudioStarted", "localAudioStopped", "localAudioFailed",
  "localVideoStarted", "localVideoStopped", "localVideoFailed",
  "localScreenStarted", "localScreenStopped", "localScreenFailed",
  "localScreenAudioFailed", "localScreenAudioStopped",
  "remoteLiveMediaTrackStarted", "remoteLiveMediaTrackStopped",
  "remoteAudioTrackStarted",
  "localMuteWasSet", "localDeafenWasSet",
  "setSpeaking", "setHandRaised",
  "setAdminMute", "setAdminDeafen", "kickPeer",
  "getUserProfile", "getUserProfiles",
  "viewProfileMenu", "viewContextMenu",
  "log",
];

function makePatch(
  bridge: "nativeToWebRtc" | "webRtcToNative",
  method: string,
): Patch {
  return {
    bridge,
    method,
    before(args) {
      const ts = new Date().toLocaleTimeString("en-US", { hour12: false });
      const argsStr = args.length > 0
        ? args.map((a) => {
            try { return JSON.stringify(a); }
            catch { return String(a); }
          }).join(", ")
        : "";
      const direction = bridge === "nativeToWebRtc" ? "N->W" : "W->N";
      nativeLog(`[${ts}] ${direction} ${method}(${argsStr.slice(0, 200)})`);
    },
  };
}

export default {
  name: "bridge-event-logger",
  description: "Logs all bridge events for debugging",
  version: "0.1.0",
  authors: [{ name: "YourName" }],

  settings: {
    logNativeToWebRtc: {
      type: "boolean",
      default: true,
      description: "Log Native -> WebRTC bridge calls",
    },
    logWebRtcToNative: {
      type: "boolean",
      default: true,
      description: "Log WebRTC -> Native bridge calls",
    },
    skipNoisy: {
      type: "boolean",
      default: true,
      description: "Skip high-frequency methods (receiveRawPacket, setSpeaking, etc.)",
    },
  },

  patches: [
    ...nativeToWebRtcMethods.map((m) => makePatch("nativeToWebRtc", m)),
    ...webRtcToNativeMethods.map((m) => makePatch("webRtcToNative", m)),
  ],

  start() {
    nativeLog(`Bridge Event Logger active — monitoring ${this.patches!.length} methods`);
    nativeLog("Tip: Use settings to filter which directions are logged.");
  },

  stop() {
    nativeLog("Bridge Event Logger stopped");
  },
} satisfies UprootedPlugin;
```

**Key concepts:** Dynamic patch generation for all known methods, `JSON.stringify` for readable arg output, configurable settings to reduce noise. The full bridge method list comes from [BRIDGE_REFERENCE.md](BRIDGE_REFERENCE.md).

> **Note:** This plugin generates a lot of log output. The `receiveRawPacket` and `setSpeaking` methods fire very frequently during calls. Consider filtering them out (check the `skipNoisy` setting pattern) for normal use.

---

## Anti-Kick

**Difficulty: Beginner**

Uses a `before` handler that returns `false` to cancel bridge calls. Blocks both directions of kick commands.

```typescript
import type { UprootedPlugin } from "../../types/plugin.js";
import { nativeLog } from "../../api/native.js";

export default {
  name: "anti-kick",
  description: "Blocks kick commands (both directions)",
  version: "0.1.0",
  authors: [{ name: "YourName" }],

  patches: [
    {
      bridge: "nativeToWebRtc",
      method: "kick",
      before(args) {
        nativeLog(`Blocked outgoing kick for user: ${args[0]}`);
        return false; // Cancel the kick
      },
    },
    {
      bridge: "webRtcToNative",
      method: "kickPeer",
      before(args) {
        nativeLog(`Blocked incoming kick request for user: ${args[0]}`);
        return false;
      },
    },
  ],
} satisfies UprootedPlugin;
```

**Key concepts:** Returning `false` from `before` to cancel, patching both bridge directions, different method names for the same action (`kick` vs `kickPeer`). See [Bridge Reference -- kick](BRIDGE_REFERENCE.md#kickuserid) and [Bridge Reference -- kickPeer](BRIDGE_REFERENCE.md#kickpeeruserid).

---

## Voice Activity Monitor

**Difficulty: Intermediate**

Monitors speaking events and injects a CSS indicator showing who's talking. Demonstrates `before` handlers, CSS injection, and DOM manipulation.

```typescript
import type { UprootedPlugin } from "../../types/plugin.js";
import type { UserGuid, DeviceGuid } from "../../types/bridge.js";
import { injectCss, removeCss } from "../../api/css.js";

let speakingUsers = new Map<string, string>(); // userId -> deviceId
let indicator: HTMLDivElement | null = null;

function updateIndicator(): void {
  if (!indicator) return;
  if (speakingUsers.size === 0) {
    indicator.style.display = "none";
    return;
  }
  indicator.style.display = "block";
  const users = Array.from(speakingUsers.keys());
  indicator.textContent = `Speaking: ${users.join(", ")}`;
}

export default {
  name: "voice-monitor",
  description: "Shows a live indicator of who is speaking",
  version: "0.1.0",
  authors: [{ name: "YourName" }],

  css: `
    #voice-monitor-indicator {
      position: fixed;
      top: 8px;
      left: 50%;
      transform: translateX(-50%);
      z-index: 999999;
      padding: 6px 16px;
      background: rgba(45, 125, 70, 0.9);
      color: #fff;
      font: 12px monospace;
      border-radius: 20px;
      pointer-events: none;
      transition: opacity 0.2s;
    }
  `,

  patches: [
    {
      bridge: "webRtcToNative",
      method: "setSpeaking",
      before(args) {
        const [isSpeaking, deviceId, userId] = args as [boolean, DeviceGuid, UserGuid];
        if (isSpeaking) {
          speakingUsers.set(userId, deviceId);
        } else {
          speakingUsers.delete(userId);
        }
        updateIndicator();
      },
    },
  ],

  start() {
    indicator = document.createElement("div");
    indicator.id = "voice-monitor-indicator";
    indicator.style.display = "none";
    document.body.appendChild(indicator);
  },

  stop() {
    indicator?.remove();
    indicator = null;
    speakingUsers.clear();
  },
} satisfies UprootedPlugin;
```

**Key concepts:** Using `css` field + DOM elements together, tracking state across events, cleaning up in `stop()`. The `setSpeaking` bridge method is documented in [Bridge Reference -- setSpeaking](BRIDGE_REFERENCE.md#setspeakingisspeaking-deviceid-userid).

---

## Custom Theme

**Difficulty: Beginner**

Applies a custom color scheme by overriding Root's CSS variables. Demonstrates the native API for theming.

```typescript
import type { UprootedPlugin } from "../../types/plugin.js";
import { setCssVariables, removeCssVariable } from "../../api/native.js";

// All the variables we override (for cleanup)
const THEME_VARS: Record<string, string> = {
  "--rootsdk-brand-primary": "#e11d48",
  "--rootsdk-brand-secondary": "#fb7185",
  "--rootsdk-brand-tertiary": "#be123c",
  "--rootsdk-background-primary": "#1a1a2e",
  "--rootsdk-background-secondary": "#22223b",
  "--rootsdk-background-tertiary": "#16161a",
  "--rootsdk-input": "#16161a",
  "--rootsdk-border": "#3a3a5c",
  "--rootsdk-link": "#fb923c",
  "--rootsdk-muted": "#4a4a6a",
};

export default {
  name: "rose-theme",
  description: "A rose/purple custom theme",
  version: "0.1.0",
  authors: [{ name: "YourName" }],

  start() {
    setCssVariables(THEME_VARS);
  },

  stop() {
    for (const name of Object.keys(THEME_VARS)) {
      removeCssVariable(name);
    }
  },
} satisfies UprootedPlugin;
```

**Key concepts:** `setCssVariables` for batch overrides, `removeCssVariable` for cleanup, using `--rootsdk-*` override pattern.

See [ROOT_ENVIRONMENT.md](ROOT_ENVIRONMENT.md#css-variable-system) for the full list of overridable variables and the two-layer variable architecture.

---

## Settings Example

**Difficulty: Intermediate**

Demonstrates all four setting types (boolean, string, number, select) and reading them at runtime.

```typescript
import type { UprootedPlugin } from "../../types/plugin.js";
import { nativeLog } from "../../api/native.js";
import { injectCss, removeCss } from "../../api/css.js";

export default {
  name: "settings-demo",
  description: "Demonstrates all setting types",
  version: "0.1.0",
  authors: [{ name: "YourName" }],

  settings: {
    enabled: {
      type: "boolean",
      default: true,
      description: "Enable the visual overlay",
    },
    label: {
      type: "string",
      default: "Uprooted",
      description: "Text shown in the overlay",
    },
    opacity: {
      type: "number",
      default: 80,
      description: "Overlay opacity (0-100)",
      min: 0,
      max: 100,
    },
    position: {
      type: "select",
      default: "bottom-right",
      description: "Overlay position",
      options: ["top-left", "top-right", "bottom-left", "bottom-right"],
    },
  },

  start() {
    // Read settings with fallbacks
    const config = window.__UPROOTED_SETTINGS__?.plugins?.["settings-demo"]?.config;
    const enabled = (config?.enabled as boolean) ?? true;
    const label = (config?.label as string) ?? "Uprooted";
    const opacity = (config?.opacity as number) ?? 80;
    const position = (config?.position as string) ?? "bottom-right";

    nativeLog(`Settings Demo: enabled=${enabled}, label="${label}", opacity=${opacity}, pos=${position}`);

    if (!enabled) return;

    const [v, h] = position.split("-");

    injectCss("settings-demo-overlay", `
      #settings-demo-overlay {
        position: fixed;
        ${v}: 12px;
        ${h}: 12px;
        z-index: 999999;
        padding: 6px 12px;
        background: rgba(45, 125, 70, ${opacity / 100});
        color: #fff;
        font: 12px sans-serif;
        border-radius: 6px;
        pointer-events: none;
      }
    `);

    const overlay = document.createElement("div");
    overlay.id = "settings-demo-overlay";
    overlay.textContent = label;
    document.body.appendChild(overlay);
  },

  stop() {
    document.getElementById("settings-demo-overlay")?.remove();
    removeCss("settings-demo-overlay");
  },
} satisfies UprootedPlugin;
```

**Key concepts:** All four `SettingField` types (defined in `src/types/plugin.ts:23`), reading config from `__UPROOTED_SETTINGS__`, fallback defaults, dynamic CSS based on settings. See [API Reference -- Settings Definition](API_REFERENCE.md#settings-definition).

---

## DOM Injector

**Difficulty: Intermediate**

Waits for a specific DOM element, injects custom content, and uses MutationObserver to re-inject if Root removes it. Demonstrates the DOM API.

```typescript
import type { UprootedPlugin } from "../../types/plugin.js";
import { waitForElement, observe } from "../../api/dom.js";
import { nativeLog } from "../../api/native.js";

let disconnect: (() => void) | null = null;

function createBadge(): HTMLDivElement {
  const badge = document.createElement("div");
  badge.id = "dom-injector-badge";
  badge.textContent = "Modded";
  badge.style.cssText =
    "display: inline-flex; align-items: center; padding: 2px 8px; " +
    "background: #2D7D46; color: #fff; font: 10px sans-serif; " +
    "border-radius: 4px; margin-left: 8px;";
  return badge;
}

export default {
  name: "dom-injector",
  description: "Injects a 'Modded' badge next to the app title",
  version: "0.1.0",
  authors: [{ name: "YourName" }],

  async start() {
    try {
      // Wait for the title element to appear (up to 15 seconds)
      const title = await waitForElement<HTMLElement>("h1, [class*='title']", 15000);

      // Inject badge
      const injectBadge = () => {
        if (document.getElementById("dom-injector-badge")) return; // Already there
        title.parentElement?.appendChild(createBadge());
      };

      injectBadge();

      // Watch for React re-renders that might remove our badge
      if (title.parentElement) {
        disconnect = observe(title.parentElement, () => {
          if (!document.getElementById("dom-injector-badge")) {
            nativeLog("Badge was removed by React, re-injecting");
            injectBadge();
          }
        });
      }
    } catch (err) {
      nativeLog(`DOM Injector failed: ${err}`);
    }
  },

  stop() {
    disconnect?.();
    disconnect = null;
    document.getElementById("dom-injector-badge")?.remove();
  },
} satisfies UprootedPlugin;
```

**Key concepts:** `waitForElement` with timeout (defined in `src/api/dom.ts:9`), `observe` for re-injection on React re-renders (defined in `src/api/dom.ts:44`), cleanup in `stop()`, error handling with `nativeLog`. See [Getting Started -- Tutorial 5: DOM Injection](GETTING_STARTED.md#tutorial-5-dom-injection) for a detailed walkthrough.

---

## Notification Interceptor

**Difficulty: Intermediate**

Intercepts bridge events related to user state changes and displays custom on-screen notifications. Demonstrates multi-method patching, DOM creation, and timed cleanup.

```typescript
import type { UprootedPlugin } from "../../types/plugin.js";
import type { Patch } from "../../types/plugin.js";
import type { UserGuid, DeviceGuid } from "../../types/bridge.js";
import { injectCss, removeCss } from "../../api/css.js";

let notificationContainer: HTMLDivElement | null = null;
let notificationId = 0;

function showNotification(message: string, color = "#2D7D46"): void {
  if (!notificationContainer) return;

  const id = ++notificationId;
  const toast = document.createElement("div");
  toast.className = "uprooted-notification-toast";
  toast.id = `uprooted-toast-${id}`;
  toast.style.borderLeftColor = color;
  toast.textContent = message;

  notificationContainer.appendChild(toast);

  // Auto-remove after 4 seconds with a fade-out
  setTimeout(() => {
    toast.style.opacity = "0";
    toast.style.transform = "translateX(120%)";
    setTimeout(() => toast.remove(), 300);
  }, 4000);
}

export default {
  name: "notification-interceptor",
  description: "Shows on-screen notifications for user state changes",
  version: "0.1.0",
  authors: [{ name: "YourName" }],

  css: `
    #uprooted-notification-container {
      position: fixed;
      top: 12px;
      right: 12px;
      z-index: 999999;
      display: flex;
      flex-direction: column;
      gap: 8px;
      pointer-events: none;
      max-width: 320px;
    }

    .uprooted-notification-toast {
      padding: 10px 16px;
      background: var(--color-background-secondary, #121A26);
      color: var(--color-text-primary, #F2F2F2);
      font: 13px sans-serif;
      border-radius: 8px;
      border-left: 4px solid #2D7D46;
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
      transition: opacity 0.3s, transform 0.3s;
      pointer-events: auto;
    }
  `,

  settings: {
    showJoinLeave: {
      type: "boolean",
      default: true,
      description: "Show notifications when users join or leave",
    },
    showMuteDeafen: {
      type: "boolean",
      default: true,
      description: "Show notifications for mute/deafen state changes",
    },
    showModeration: {
      type: "boolean",
      default: true,
      description: "Show notifications for moderation actions (kick, admin mute)",
    },
  },

  patches: [
    // Speaking start/stop (used to detect join/leave presence)
    {
      bridge: "webRtcToNative",
      method: "setSpeaking",
      before(args) {
        const [isSpeaking, , userId] = args as [boolean, DeviceGuid, UserGuid];
        if (isSpeaking) {
          showNotification(`${userId.slice(0, 8)}... started speaking`);
        }
      },
    },

    // Theme changes
    {
      bridge: "nativeToWebRtc",
      method: "setTheme",
      before(args) {
        showNotification(`Theme changed to: ${args[0]}`, "#3B6AF8");
      },
    },

    // Mute/deafen
    {
      bridge: "nativeToWebRtc",
      method: "setMute",
      before(args) {
        const isMuted = args[0] as boolean;
        showNotification(isMuted ? "You are now muted" : "You are now unmuted", "#E88F3D");
      },
    },
    {
      bridge: "nativeToWebRtc",
      method: "setDeafen",
      before(args) {
        const isDeafened = args[0] as boolean;
        showNotification(isDeafened ? "You are now deafened" : "You are now undeafened", "#E88F3D");
      },
    },

    // Moderation actions
    {
      bridge: "nativeToWebRtc",
      method: "kick",
      before(args) {
        showNotification(`Kick sent for user: ${String(args[0]).slice(0, 8)}...`, "#F03F36");
      },
    },
    {
      bridge: "nativeToWebRtc",
      method: "setAdminMute",
      before(args) {
        const [userId, isMuted] = args as [UserGuid, boolean];
        showNotification(
          `Admin ${isMuted ? "muted" : "unmuted"}: ${userId.slice(0, 8)}...`,
          "#F03F36",
        );
      },
    },

    // Session lifecycle
    {
      bridge: "webRtcToNative",
      method: "initialized",
      before() {
        showNotification("Voice session connected", "#49D6AC");
      },
    },
    {
      bridge: "webRtcToNative",
      method: "disconnected",
      before() {
        showNotification("Voice session disconnected", "#F03F36");
      },
    },
  ],

  start() {
    notificationContainer = document.createElement("div");
    notificationContainer.id = "uprooted-notification-container";
    document.body.appendChild(notificationContainer);
  },

  stop() {
    notificationContainer?.remove();
    notificationContainer = null;
    notificationId = 0;
  },
} satisfies UprootedPlugin;
```

**Key concepts:** Multi-method patching across both bridges, dynamic DOM element creation for toast notifications, CSS using Root's theme variables (`var(--color-background-secondary)`), auto-cleanup with `setTimeout`, plugin settings for filtering notification types.

---

## Call Logger

**Difficulty: Advanced**

Monitors multiple bridge methods to log the full lifecycle of a voice call. Demonstrates multi-method patching with dynamic patch generation.

```typescript
import type { UprootedPlugin } from "../../types/plugin.js";
import type { Patch } from "../../types/plugin.js";
import type { InitializeDesktopWebRtcPayload, Theme } from "../../types/bridge.js";
import { nativeLog } from "../../api/native.js";

// Format a timestamp for log lines
function ts(): string {
  return new Date().toLocaleTimeString("en-US", { hour12: false });
}

// Build patches dynamically for cleaner code
function logPatch(
  bridge: "nativeToWebRtc" | "webRtcToNative",
  method: string,
  format?: (args: unknown[]) => string,
): Patch {
  return {
    bridge,
    method,
    before(args) {
      const detail = format ? format(args) : args.map(String).join(", ");
      nativeLog(`[${ts()}] ${bridge}.${method}(${detail})`);
    },
  };
}

export default {
  name: "call-logger",
  description: "Logs the full voice call lifecycle",
  version: "0.1.0",
  authors: [{ name: "YourName" }],

  patches: [
    // Session lifecycle
    logPatch("nativeToWebRtc", "initialize", (args) => {
      const s = args[0] as InitializeDesktopWebRtcPayload;
      return `channel=${s.channelId}, user=${s.userId}`;
    }),
    logPatch("nativeToWebRtc", "disconnect"),
    logPatch("webRtcToNative", "initialized"),
    logPatch("webRtcToNative", "disconnected"),
    logPatch("webRtcToNative", "failed", (args) => JSON.stringify(args[0])),

    // Media toggles
    logPatch("nativeToWebRtc", "setIsAudioOn"),
    logPatch("nativeToWebRtc", "setIsVideoOn"),
    logPatch("nativeToWebRtc", "setIsScreenShareOn"),

    // User state
    logPatch("nativeToWebRtc", "setMute"),
    logPatch("nativeToWebRtc", "setDeafen"),
    logPatch("nativeToWebRtc", "setHandRaised"),
    logPatch("nativeToWebRtc", "setTheme", (args) => args[0] as string),

    // Remote events
    logPatch("webRtcToNative", "setSpeaking", (args) => {
      const [speaking, , userId] = args;
      return `${userId} ${speaking ? "started" : "stopped"} speaking`;
    }),

    // Moderation
    logPatch("nativeToWebRtc", "kick"),
    logPatch("nativeToWebRtc", "setAdminMute"),
    logPatch("nativeToWebRtc", "setAdminDeafen"),
  ],

  start() {
    nativeLog(`[${ts()}] Call Logger active — monitoring ${this.patches!.length} methods`);
  },

  stop() {
    nativeLog(`[${ts()}] Call Logger stopped`);
  },
} satisfies UprootedPlugin;
```

**Key concepts:** Dynamic patch generation with a helper function, formatting args per-method, monitoring both bridge directions, timestamp logging, using `this.patches` to reference own config. For the full list of interceptable methods, see [BRIDGE_REFERENCE.md](BRIDGE_REFERENCE.md).

---

## CSS Theme Switcher

**Difficulty: Advanced**

A plugin that provides hotkey-driven theme switching with smooth CSS transitions. Demonstrates CSS variable overrides, bridge interception for theme sync, and keyboard event handling.

```typescript
import type { UprootedPlugin } from "../../types/plugin.js";
import type { Theme } from "../../types/bridge.js";
import { setCssVariables, removeCssVariable, getCurrentTheme } from "../../api/native.js";
import { injectCss, removeCss } from "../../api/css.js";
import { nativeLog } from "../../api/native.js";

// Define theme presets with full variable overrides
const THEME_PRESETS: Record<string, Record<string, string>> = {
  midnight: {
    "--rootsdk-brand-primary": "#6366f1",
    "--rootsdk-brand-secondary": "#a5b4fc",
    "--rootsdk-brand-tertiary": "#4f46e5",
    "--rootsdk-background-primary": "#0f0f23",
    "--rootsdk-background-secondary": "#1a1a3e",
    "--rootsdk-background-tertiary": "#0a0a1a",
    "--rootsdk-input": "#0a0a1a",
    "--rootsdk-border": "#2a2a5c",
    "--rootsdk-link": "#818cf8",
    "--rootsdk-muted": "#4a4a6a",
  },
  forest: {
    "--rootsdk-brand-primary": "#22c55e",
    "--rootsdk-brand-secondary": "#86efac",
    "--rootsdk-brand-tertiary": "#16a34a",
    "--rootsdk-background-primary": "#0a1f0a",
    "--rootsdk-background-secondary": "#132613",
    "--rootsdk-background-tertiary": "#061206",
    "--rootsdk-input": "#061206",
    "--rootsdk-border": "#1e3a1e",
    "--rootsdk-link": "#4ade80",
    "--rootsdk-muted": "#3a5a3a",
  },
  sunset: {
    "--rootsdk-brand-primary": "#f97316",
    "--rootsdk-brand-secondary": "#fdba74",
    "--rootsdk-brand-tertiary": "#ea580c",
    "--rootsdk-background-primary": "#1c1008",
    "--rootsdk-background-secondary": "#291a0e",
    "--rootsdk-background-tertiary": "#120a04",
    "--rootsdk-input": "#120a04",
    "--rootsdk-border": "#3d2a16",
    "--rootsdk-link": "#fb923c",
    "--rootsdk-muted": "#5c4a2a",
  },
};

const ALL_VAR_NAMES = new Set<string>();
for (const vars of Object.values(THEME_PRESETS)) {
  for (const name of Object.keys(vars)) {
    ALL_VAR_NAMES.add(name);
  }
}

const presetNames = Object.keys(THEME_PRESETS);
let currentPresetIndex = -1; // -1 means "no custom preset active"
let keydownHandler: ((e: KeyboardEvent) => void) | null = null;

function clearCustomTheme(): void {
  for (const name of ALL_VAR_NAMES) {
    removeCssVariable(name);
  }
  currentPresetIndex = -1;
}

function applyPreset(index: number): void {
  const name = presetNames[index];
  if (!name) return;

  clearCustomTheme();
  currentPresetIndex = index;
  setCssVariables(THEME_PRESETS[name]);
  nativeLog(`Theme Switcher: applied "${name}" theme`);
}

export default {
  name: "css-theme-switcher",
  description: "Hotkey-driven theme switcher with smooth transitions (Ctrl+Shift+T)",
  version: "0.1.0",
  authors: [{ name: "YourName" }],

  css: `
    /* Smooth transitions when switching themes */
    :root {
      transition: --rootsdk-brand-primary 0.3s,
                  --rootsdk-background-primary 0.3s,
                  --rootsdk-background-secondary 0.3s;
    }
    body, body * {
      transition: background-color 0.3s ease, color 0.2s ease, border-color 0.2s ease;
    }
  `,

  patches: [
    {
      bridge: "nativeToWebRtc",
      method: "setTheme",
      before(args) {
        // When Root changes theme, clear our custom overrides so they
        // don't conflict with Root's built-in theme
        if (currentPresetIndex >= 0) {
          clearCustomTheme();
          nativeLog("Theme Switcher: cleared custom theme (Root theme changed)");
        }
      },
    },
  ],

  start() {
    keydownHandler = (e: KeyboardEvent) => {
      // Ctrl+Shift+T cycles through presets, Ctrl+Shift+0 resets
      if (e.ctrlKey && e.shiftKey && e.key === "T") {
        e.preventDefault();
        const nextIndex = (currentPresetIndex + 1) % presetNames.length;
        applyPreset(nextIndex);
      } else if (e.ctrlKey && e.shiftKey && e.key === "0") {
        e.preventDefault();
        clearCustomTheme();
        nativeLog("Theme Switcher: reset to default");
      }
    };

    document.addEventListener("keydown", keydownHandler);
    nativeLog(`Theme Switcher active — Ctrl+Shift+T to cycle (${presetNames.length} presets), Ctrl+Shift+0 to reset`);
  },

  stop() {
    if (keydownHandler) {
      document.removeEventListener("keydown", keydownHandler);
      keydownHandler = null;
    }
    clearCustomTheme();
  },
} satisfies UprootedPlugin;
```

**Key concepts:** Multiple theme preset definitions using `--rootsdk-*` variables, keyboard event handling with cleanup, bridge patch to detect when Root changes theme (so custom overrides don't conflict), CSS transition injection for smooth theme changes, full cleanup in `stop()`. See [Root Environment -- CSS Variable System](ROOT_ENVIRONMENT.md#css-variable-system) for how the variable override system works.

---

**Canonical for:** 11 working code examples, copy-paste plugin starters, pattern demonstrations
**Not canonical for:** API contracts → [API_REFERENCE.md](API_REFERENCE.md) | tutorial → [GETTING_STARTED.md](GETTING_STARTED.md) | advanced patterns → [ADVANCED_DEVELOPMENT.md](ADVANCED_DEVELOPMENT.md)
*Plugin examples. Last updated 2026-02-19.*
