# examples

copy-paste plugin examples.

## minimal template

bare minimum plugin:

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

register in `src/core/preload.ts` with `loader.register(myPlugin)`.

## theme logger

logs theme changes via bridge interception:

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

## bridge event logger

logs all bridge events for debugging:

```typescript
import type { UprootedPlugin } from "../../types/plugin.js";
import type { Patch } from "../../types/plugin.js";
import { nativeLog } from "../../api/native.js";

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
    nativeLog(`Bridge Event Logger active - monitoring ${this.patches!.length} methods`);
    nativeLog("Tip: Use settings to filter which directions are logged.");
  },

  stop() {
    nativeLog("Bridge Event Logger stopped");
  },
} satisfies UprootedPlugin;
```

generates a lot of output. `receiveRawPacket` and `setSpeaking` fire constantly during calls.

## anti-kick

blocks kick commands in both directions:

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
        return false;
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

## voice activity monitor

tracks who's speaking with a live indicator:

```typescript
import type { UprootedPlugin } from "../../types/plugin.js";
import type { UserGuid, DeviceGuid } from "../../types/bridge.js";
import { injectCss, removeCss } from "../../api/css.js";

let speakingUsers = new Map<string, string>();
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

## custom theme

applies css variable overrides for a custom color scheme:

```typescript
import type { UprootedPlugin } from "../../types/plugin.js";
import { setCssVariables, removeCssVariable } from "../../api/native.js";

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

## settings example

all four setting types with runtime reading:

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

## dom injector

waits for a DOM element, injects content, re-injects on re-renders:

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
      const title = await waitForElement<HTMLElement>("h1, [class*='title']", 15000);
      const injectBadge = () => {
        if (document.getElementById("dom-injector-badge")) return;
        title.parentElement?.appendChild(createBadge());
      };
      injectBadge();
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

## notification interceptor

custom on-screen toast notifications for bridge events:

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
    {
      bridge: "nativeToWebRtc",
      method: "setTheme",
      before(args) {
        showNotification(`Theme changed to: ${args[0]}`, "#3B6AF8");
      },
    },
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

## call logger

monitors bridge methods for the full voice call lifecycle:

```typescript
import type { UprootedPlugin } from "../../types/plugin.js";
import type { Patch } from "../../types/plugin.js";
import type { InitializeDesktopWebRtcPayload, Theme } from "../../types/bridge.js";
import { nativeLog } from "../../api/native.js";

function ts(): string {
  return new Date().toLocaleTimeString("en-US", { hour12: false });
}

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
    logPatch("nativeToWebRtc", "initialize", (args) => {
      const s = args[0] as InitializeDesktopWebRtcPayload;
      return `channel=${s.channelId}, user=${s.userId}`;
    }),
    logPatch("nativeToWebRtc", "disconnect"),
    logPatch("webRtcToNative", "initialized"),
    logPatch("webRtcToNative", "disconnected"),
    logPatch("webRtcToNative", "failed", (args) => JSON.stringify(args[0])),
    logPatch("nativeToWebRtc", "setIsAudioOn"),
    logPatch("nativeToWebRtc", "setIsVideoOn"),
    logPatch("nativeToWebRtc", "setIsScreenShareOn"),
    logPatch("nativeToWebRtc", "setMute"),
    logPatch("nativeToWebRtc", "setDeafen"),
    logPatch("nativeToWebRtc", "setHandRaised"),
    logPatch("nativeToWebRtc", "setTheme", (args) => args[0] as string),
    logPatch("webRtcToNative", "setSpeaking", (args) => {
      const [speaking, , userId] = args;
      return `${userId} ${speaking ? "started" : "stopped"} speaking`;
    }),
    logPatch("nativeToWebRtc", "kick"),
    logPatch("nativeToWebRtc", "setAdminMute"),
    logPatch("nativeToWebRtc", "setAdminDeafen"),
  ],

  start() {
    nativeLog(`[${ts()}] Call Logger active - monitoring ${this.patches!.length} methods`);
  },

  stop() {
    nativeLog(`[${ts()}] Call Logger stopped`);
  },
} satisfies UprootedPlugin;
```

## css theme switcher

hotkey-driven theme switching with smooth transitions. ctrl+shift+t cycles presets, ctrl+shift+0 resets:

```typescript
import type { UprootedPlugin } from "../../types/plugin.js";
import type { Theme } from "../../types/bridge.js";
import { setCssVariables, removeCssVariable, getCurrentTheme } from "../../api/native.js";
import { injectCss, removeCss } from "../../api/css.js";
import { nativeLog } from "../../api/native.js";

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
let currentPresetIndex = -1;
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
        if (currentPresetIndex >= 0) {
          clearCustomTheme();
          nativeLog("Theme Switcher: cleared custom theme (app theme changed)");
        }
      },
    },
  ],

  start() {
    keydownHandler = (e: KeyboardEvent) => {
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
    nativeLog(`Theme Switcher active - Ctrl+Shift+T to cycle (${presetNames.length} presets), Ctrl+Shift+0 to reset`);
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
