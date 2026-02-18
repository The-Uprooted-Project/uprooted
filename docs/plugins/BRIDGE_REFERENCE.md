# Bridge Reference

Complete reference for all 71 bridge methods across Root's two WebRTC bridge interfaces. These are the methods you can intercept with [Patch](API_REFERENCE.md#patch-interface) definitions in your plugin.

> **Related Docs:**
> [API Reference](API_REFERENCE.md) |
> [Root Environment](ROOT_ENVIRONMENT.md) |
> [TypeScript Reference](../framework/TYPESCRIPT_REFERENCE.md) |
> [Architecture](../framework/ARCHITECTURE.md)

## Table of Contents

- [How Interception Works](#how-interception-works)
- [BridgeEvent Structure](#bridgeevent-structure)
- [Type Definitions](#type-definitions)
- [INativeToWebRtc (42 methods)](#inativetowebrtc--native--webrtc)
  - [Connection](#connection)
  - [Media Control](#media-control)
  - [Device Selection](#device-selection)
  - [User State](#user-state)
  - [UI & Quality](#ui--quality)
  - [Moderation](#moderation)
  - [Volume Control](#volume-control)
  - [Packets & Native Audio](#packets--native-audio)
- [IWebRtcToNative (32 methods)](#iwebrtctonative--webrtc--native)
  - [Session Lifecycle](#session-lifecycle)
  - [Local Track Events](#local-track-events)
  - [Remote Track Events](#remote-track-events)
  - [User State Notifications](#user-state-notifications)
  - [Moderation Outbound](#moderation-outbound)
  - [Profile & UI](#profile--ui)
  - [Logging](#logging)
- [Common Interception Patterns](#common-interception-patterns)

---

## How Interception Works

Root exposes two bridge objects on `window`:

| Global | Direction | Description |
|--------|-----------|-------------|
| `__nativeToWebRtc` | C# host -> JS WebRTC | The .NET host calls these to control the WebRTC session (mute, theme, devices, etc.) |
| `__webRtcToNative` | JS WebRTC -> C# host | The WebRTC JS layer calls these to notify the host of state changes (speaking, track events, etc.) |

Uprooted wraps both with ES6 Proxies at startup (see `src/api/bridge.ts:49`, `installBridgeProxy()`). Every method call passes through the proxy, which:

1. Creates a `BridgeEvent` with the method name and arguments
2. Runs all registered `before` / `replace` handlers (from plugin patches)
3. If not cancelled, calls the original method

The proxy is created by `createBridgeProxy()` at `src/api/bridge.ts:21`. The event dispatch happens through `pluginLoader.emit()` at `src/api/bridge.ts:37`.

Plugins intercept these calls using `patches` in their plugin definition:

```typescript
export default {
  name: "my-plugin",
  // ...
  patches: [
    {
      bridge: "nativeToWebRtc",   // or "webRtcToNative"
      method: "setTheme",
      before(args) {
        console.log("Theme is changing to:", args[0]);
      }
    }
  ]
} satisfies UprootedPlugin;
```

---

## BridgeEvent Structure

Defined in `src/core/pluginLoader.ts:11`.

```typescript
interface BridgeEvent {
  method: string;        // Method name (e.g. "setTheme", "disconnect")
  args: unknown[];       // Arguments array
  cancelled: boolean;    // true = original method will NOT be called
  returnValue?: unknown; // Custom return value when cancelled via replace
}
```

---

## Type Definitions

These types are used across bridge method signatures. Defined in `src/types/bridge.ts`.

```typescript
// Opaque branded types for Root's GUID system
type UserGuid = string & { readonly __brand: "UserGuid" };
type DeviceGuid = string & { readonly __brand: "DeviceGuid" };

type TileType = "camera" | "screen" | "audio";
type Theme = "dark" | "light" | "pure-dark";
type ScreenQualityMode = "motion" | "detail" | "auto";
type Codec = string;

interface Coordinates {
  x: number;
  y: number;
}

interface IUserResponse {
  userId: UserGuid;
  displayName: string;
  avatarUrl?: string;
  [key: string]: unknown;
}

type WebRtcPermission = Record<string, boolean>;

interface UserMediaStreamConstraints {
  audio?: MediaTrackConstraints | boolean;
  video?: MediaTrackConstraints | boolean;
}

interface DisplayMediaStreamConstraints {
  audio?: MediaTrackConstraints | boolean;
  video?: MediaTrackConstraints | boolean;
}

interface VolumeBoosterSettings {
  enabled: boolean;
  gain: number;
}

interface WebRtcError {
  code: string;
  message: string;
}

interface InitializeDesktopWebRtcPayload {
  token: string;
  channelId: string;
  communityId: string;
  userId: UserGuid;
  deviceId: DeviceGuid;
  theme: Theme;
  [key: string]: unknown;
}

interface IPacket {
  type: string;
  data: unknown;
}
```

> **Note:** `UserGuid` and `DeviceGuid` are branded string types. At runtime they're just strings, but TypeScript will enforce you don't mix them up.

---

## INativeToWebRtc -- Native -> WebRTC

**42 methods.** These are called by Root's C# host to control the WebRTC session. Intercept with `bridge: "nativeToWebRtc"`.

Source: Interface defined in `src/types/bridge.ts`. Proxy wrapping in `src/api/bridge.ts:56`.

### Connection

#### `initialize(state)`
Start a WebRTC session. This is the first call when joining a voice channel.

```typescript
initialize(state: InitializeDesktopWebRtcPayload): void
```

The payload contains the auth token, channel/community IDs, user/device IDs, and the current theme. This is rich data for plugins that want to know session context.

| Payload Field | Type | Description |
|---------------|------|-------------|
| `token` | `string` | Bearer auth token for the session |
| `channelId` | `string` | Voice channel being joined |
| `communityId` | `string` | Community (server) the channel belongs to |
| `userId` | `UserGuid` | Current user's ID |
| `deviceId` | `DeviceGuid` | Current device's ID |
| `theme` | `Theme` | Active theme at join time |

#### `disconnect()`
End the WebRTC session. Called when leaving a voice channel.

```typescript
disconnect(): void
```

---

### Media Control

#### `setIsVideoOn(isVideo)`
Toggle the local camera.

```typescript
setIsVideoOn(isVideo: boolean): void
```

#### `setIsScreenShareOn(isScreenShare, withAudio?)`
Toggle screen sharing, optionally including system audio.

```typescript
setIsScreenShareOn(isScreenShare: boolean, withAudio?: boolean): void
```

#### `setIsAudioOn(isAudio)`
Toggle the local microphone.

```typescript
setIsAudioOn(isAudio: boolean): void
```

---

### Device Selection

#### `updateVideoDeviceId(videoSourceId)`
Switch the active camera device.

```typescript
updateVideoDeviceId(videoSourceId: string): void
```

#### `updateAudioInputDeviceId(micSourceId)`
Switch the active microphone.

```typescript
updateAudioInputDeviceId(micSourceId: string): void
```

#### `updateAudioOutputDeviceId(soundSourceId)`
Switch the active speaker/headphones.

```typescript
updateAudioOutputDeviceId(soundSourceId: string): void
```

#### `updateScreenShareDeviceId(screenSourceId)`
Switch which screen/window is being shared.

```typescript
updateScreenShareDeviceId(screenSourceId: string): void
```

#### `updateScreenAudioDeviceId(screenAudioSourceId)`
Switch the audio source for screen share.

```typescript
updateScreenAudioDeviceId(screenAudioSourceId: string): void
```

---

### User State

#### `updateProfile(user)`
Push updated profile data to the WebRTC layer.

```typescript
updateProfile(user: IUserResponse): void
```

#### `updateMyPermission(myUserPermission)`
Update the local user's permission set.

```typescript
updateMyPermission(myUserPermission: WebRtcPermission): void
```

#### `setPushToTalkMode(isPushToTalkMode)`
Toggle push-to-talk mode on/off.

```typescript
setPushToTalkMode(isPushToTalkMode: boolean): void
```

#### `setPushToTalk(isPushingToTalk)`
Set the push-to-talk key state (pressed/released).

```typescript
setPushToTalk(isPushingToTalk: boolean): void
```

#### `setMute(isMuted)`
Mute or unmute the local user.

```typescript
setMute(isMuted: boolean): void
```

#### `setDeafen(isDeafened)`
Deafen or undeafen the local user.

```typescript
setDeafen(isDeafened: boolean): void
```

#### `setHandRaised(isHandRaised)`
Raise or lower the local user's hand.

```typescript
setHandRaised(isHandRaised: boolean): void
```

---

### UI & Quality

#### `setTheme(theme)`
Change the active theme. Called when the user switches themes in Root's settings.

```typescript
setTheme(theme: Theme): void  // "dark" | "light" | "pure-dark"
```

#### `setNoiseGateThreshold(threshold)`
Set the noise gate sensitivity.

```typescript
setNoiseGateThreshold(threshold: number): void
```

#### `setDenoisePower(power)`
Set the noise suppression strength.

```typescript
setDenoisePower(power: number): void
```

#### `setScreenQualityMode(qualityMode)`
Set screen share quality preference.

```typescript
setScreenQualityMode(qualityMode: ScreenQualityMode): void  // "motion" | "detail" | "auto"
```

#### `toggleFullFocus(isFullFocus)`
Toggle full-focus mode (maximizes a single user's tile).

```typescript
toggleFullFocus(isFullFocus: boolean): void
```

#### `setPreferredCodecs(preferredCodecs)`
Set preferred audio/video codecs.

```typescript
setPreferredCodecs(preferredCodecs: Codec[]): void
```

#### `setUserMediaConstraints(constraints)`
Set constraints for getUserMedia (camera/mic).

```typescript
setUserMediaConstraints(constraints: UserMediaStreamConstraints): void
```

#### `setDisplayMediaConstraints(constraints)`
Set constraints for getDisplayMedia (screen share).

```typescript
setDisplayMediaConstraints(constraints: DisplayMediaStreamConstraints): void
```

#### `setScreenContentHint(contentHint)`
Hint the content type of the screen share (e.g. "text", "motion", "detail").

```typescript
setScreenContentHint(contentHint: string): void
```

#### `screenPickerDismissed()`
Notify that the screen picker dialog was closed without selecting a source.

```typescript
screenPickerDismissed(): void
```

---

### Moderation

#### `setAdminMute(userId, isAdminMuted)`
Admin-mute another user.

```typescript
setAdminMute(userId: UserGuid, isAdminMuted: boolean): void
```

#### `setAdminDeafen(userId, isAdminDeafened)`
Admin-deafen another user.

```typescript
setAdminDeafen(userId: UserGuid, isAdminDeafened: boolean): void
```

#### `kick(userId)`
Kick a user from the voice channel.

```typescript
kick(userId: UserGuid): void
```

---

### Volume Control

#### `setTileVolume(userId, tileType, volume)`
Set volume for a specific user's tile (camera, screen, or audio).

```typescript
setTileVolume(userId: UserGuid, tileType: TileType, volume: number): void
```

#### `setOutputVolume(volume)`
Set the global output (speaker) volume.

```typescript
setOutputVolume(volume: number): void
```

#### `setInputVolume(volume)`
Set the input (microphone) volume.

```typescript
setInputVolume(volume: number): void
```

#### `customizeVolumeBooster(settings)`
Configure the volume booster feature.

```typescript
customizeVolumeBooster(settings: VolumeBoosterSettings): void
```

`VolumeBoosterSettings`: `{ enabled: boolean, gain: number }`

---

### Packets & Native Audio

#### `receiveRawPacket(data)`
Receive a raw data packet from the native layer.

```typescript
receiveRawPacket(data: unknown): void
```

#### `receiveRawPacketContainer(data)`
Receive a raw packet container from the native layer.

```typescript
receiveRawPacketContainer(data: unknown): void
```

#### `receivePacket(packet)`
Receive a typed packet from the native layer.

```typescript
receivePacket(packet: IPacket): void  // { type: string, data: unknown }
```

#### `nativeLoopbackAudioStarted(sampleRate, channels)`
Notify that native audio loopback has started.

```typescript
nativeLoopbackAudioStarted(sampleRate: number, channels: number): Promise<void>
```

#### `receiveNativeLoopbackAudioData(bridgeData, byteCount)`
Push native loopback audio sample data to the WebRTC layer.

```typescript
receiveNativeLoopbackAudioData(bridgeData: unknown, byteCount: number): void
```

#### `getNativeLoopbackAudioTrack()`
Get the current native loopback audio track.

```typescript
getNativeLoopbackAudioTrack(): MediaStreamTrack | null
```

#### `stopNativeLoopbackAudio()`
Stop native audio loopback.

```typescript
stopNativeLoopbackAudio(): void
```

---

## IWebRtcToNative -- WebRTC -> Native

**29 methods.** These are called by Root's WebRTC JS layer to notify the C# host. Intercept with `bridge: "webRtcToNative"`.

Source: Interface defined in `src/types/bridge.ts`. Proxy wrapping in `src/api/bridge.ts:65`.

### Session Lifecycle

#### `initialized()`
Notify the host that WebRTC initialization is complete.

```typescript
initialized(): void
```

#### `disconnected()`
Notify the host that the WebRTC session has ended.

```typescript
disconnected(): void
```

#### `failed(error)`
Notify the host that the WebRTC session failed.

```typescript
failed(error: WebRtcError): void  // { code: string, message: string }
```

---

### Local Track Events

These fire when local media tracks start or stop. Each media type has a started, stopped, and failed variant.

#### Audio

```typescript
localAudioStarted(): void
localAudioStopped(): void
localAudioFailed(): void
```

#### Video

```typescript
localVideoStarted(): void
localVideoStopped(): void
localVideoFailed(): void
```

#### Screen Share

```typescript
localScreenStarted(): void
localScreenStopped(): void
localScreenFailed(): void
```

#### Screen Audio

```typescript
localScreenAudioFailed(): void
localScreenAudioStopped(): void
```

> **Note:** There is no `localScreenAudioStarted()`. Screen audio start is implied by a successful `setIsScreenShareOn(true, true)`.

---

### Remote Track Events

#### `remoteLiveMediaTrackStarted()`
A remote user started sharing live media (video or screen).

```typescript
remoteLiveMediaTrackStarted(): void
```

#### `remoteLiveMediaTrackStopped()`
A remote user stopped sharing live media.

```typescript
remoteLiveMediaTrackStopped(): void
```

#### `remoteAudioTrackStarted(userIds)`
Remote audio tracks started for the given users.

```typescript
remoteAudioTrackStarted(userIds: UserGuid[]): void
```

---

### User State Notifications

#### `localMuteWasSet(isMuted)`
The local user's mute state was actually applied.

```typescript
localMuteWasSet(isMuted: boolean): void
```

#### `localDeafenWasSet(isDeafened)`
The local user's deafen state was actually applied.

```typescript
localDeafenWasSet(isDeafened: boolean): void
```

#### `setSpeaking(isSpeaking, deviceId, userId)`
A user started or stopped speaking.

```typescript
setSpeaking(isSpeaking: boolean, deviceId: DeviceGuid, userId: UserGuid): void
```

#### `setHandRaised(isHandRaised, deviceId, userId)`
A user raised or lowered their hand.

```typescript
setHandRaised(isHandRaised: boolean, deviceId: DeviceGuid, userId: UserGuid): void
```

---

### Moderation Outbound

These notify the host when moderation actions are triggered from the WebRTC UI.

#### `setAdminMute(deviceId, isMuted)`
Request admin-mute for a device.

```typescript
setAdminMute(deviceId: DeviceGuid, isMuted: boolean): void
```

#### `setAdminDeafen(deviceId, isDeafened)`
Request admin-deafen for a device.

```typescript
setAdminDeafen(deviceId: DeviceGuid, isDeafened: boolean): void
```

#### `kickPeer(userId)`
Request to kick a user.

```typescript
kickPeer(userId: UserGuid): void
```

> **Note:** The parameter difference: `nativeToWebRtc.kick(userId)` vs `webRtcToNative.kickPeer(userId)`. Different method names, same concept, different directions.

---

### Profile & UI

#### `getUserProfile(userId)`
Request a user's profile from the native host. Returns asynchronously.

```typescript
getUserProfile(userId: UserGuid): Promise<IUserResponse>
```

#### `getUserProfiles(userIds)`
Request multiple user profiles at once.

```typescript
getUserProfiles(userIds: UserGuid[]): Promise<IUserResponse[]>
```

#### `viewProfileMenu(userId, coordinates)`
Open a user's profile popup at the given screen coordinates.

```typescript
viewProfileMenu(userId: UserGuid, coordinates: Coordinates): void
```

#### `viewContextMenu(userId, coordinates, tileType?, volume?)`
Open a user's context menu at the given screen coordinates.

```typescript
viewContextMenu(
  userId: UserGuid,
  coordinates: Coordinates,
  tileType?: TileType,
  volume?: number
): void
```

---

### Logging

#### `log(message)`
Send a log message to the native host. This is what `nativeLog()` (defined in `src/api/native.ts:42`) uses under the hood.

```typescript
log(message: string): void
```

---

## Common Interception Patterns

### Monitor Theme Changes

```typescript
patches: [
  {
    bridge: "nativeToWebRtc",
    method: "setTheme",
    before(args) {
      const theme = args[0] as Theme;
      console.log(`Theme changing to: ${theme}`);
    }
  }
]
```

### Log All Speaking Events

```typescript
patches: [
  {
    bridge: "webRtcToNative",
    method: "setSpeaking",
    before(args) {
      const [isSpeaking, deviceId, userId] = args;
      if (isSpeaking) {
        console.log(`User ${userId} started speaking`);
      }
    }
  }
]
```

### Block a Moderation Action

```typescript
patches: [
  {
    bridge: "nativeToWebRtc",
    method: "kick",
    before(args) {
      console.log("Kick blocked for user:", args[0]);
      return false;  // Cancel the call
    }
  }
]
```

### Capture Session Info on Join

```typescript
patches: [
  {
    bridge: "nativeToWebRtc",
    method: "initialize",
    before(args) {
      const state = args[0] as InitializeDesktopWebRtcPayload;
      console.log(`Joining channel ${state.channelId} in community ${state.communityId}`);
      console.log(`User: ${state.userId}, Device: ${state.deviceId}`);
    }
  }
]
```

### Force a Setting

```typescript
patches: [
  {
    bridge: "nativeToWebRtc",
    method: "setNoiseGateThreshold",
    replace(threshold) {
      // Always use our preferred threshold
      window.__nativeToWebRtc?.setNoiseGateThreshold?.(0.01);
    }
  }
]
```

> **Warning:** Be careful with `replace` on critical methods like `initialize` or `disconnect`. Replacing them incorrectly can break the WebRTC session entirely.

### Multi-Method Monitor

```typescript
const methods = ["setMute", "setDeafen", "setHandRaised"] as const;

export default {
  name: "state-logger",
  // ...
  patches: methods.map(method => ({
    bridge: "nativeToWebRtc" as const,
    method,
    before(args: unknown[]) {
      console.log(`[StateLogger] ${method}:`, args[0]);
    }
  }))
} satisfies UprootedPlugin;
```
