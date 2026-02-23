# Sentry Blocker

> **What this is:** Sentry blocker plugin reference — blocks Sentry telemetry via fetch/XHR/sendBeacon interception.

Privacy plugin that intercepts all network requests to Sentry's telemetry servers and blocks them before they leave the browser.

> **Source:** [`src/plugins/sentry-blocker/index.ts`](../../../src/plugins/sentry-blocker/index.ts)

---

## What it does

Root Communications sends telemetry to Sentry (`o4509469920133120.ingest.us.sentry.io`) with the following configuration:

| Setting | Value | Impact |
|---------|-------|--------|
| `sendDefaultPii` | `true` | Your IP address is attached to every error event |
| `replaysOnErrorSampleRate` | `0.25` | 25% of errors trigger DOM replay recordings (mouse, inputs, snapshots) |
| `tracesSampleRate` | `0.025` | Page load performance traces |
| `enableLogs` | `true` | Application logs forwarded to Sentry |
| Request breadcrumbs | enabled | Auth headers including Bearer tokens captured in request logs |

This data is sent to Sentry's servers, not Root's servers. Sentry Blocker prevents all of it from leaving.

## How it works

The plugin wraps three browser network APIs at the global level:

| API | Interception | Blocked behavior |
|-----|-------------|-----------------|
| `window.fetch()` | Checks URL before sending | Returns `new Response(null, { status: 200 })` |
| `XMLHttpRequest.open()` | Checks URL at open time | Redirects to `about:blank` |
| `navigator.sendBeacon()` | Checks URL before posting | Returns `true` (pretends success) |

URL detection uses a substring check: `url.includes("sentry.io")`. This catches all Sentry subdomains.

### Why fetch-level, not Sentry.init?

Sentry initializes at module evaluation time inside Root's JavaScript bundle -- before any plugin code runs and before the bridge is set up. By the time Uprooted starts, Sentry is already configured and queueing events. The only reliable interception point is at the network transport layer.

## Lifecycle

**start():**
1. Stores references to the original `fetch`, `XMLHttpRequest.prototype.open`, and `navigator.sendBeacon`
2. Replaces each with a wrapper that checks `isSentryUrl()` before delegating
3. Maintains a running `blockedCount` for console logging

**stop():**
1. Restores all three original functions
2. Nullifies stored references
3. Logs final blocked count

## Settings

None. The plugin is either enabled or disabled -- there is no configuration.

## Known Limitations

- **Substring detection** -- `url.includes("sentry.io")` is simple but could match unrelated domains containing that substring
- **XHR redirect** -- blocked XHR requests are redirected to `about:blank`, which causes browser warnings in the console
- **No visibility into blocked data** -- the plugin counts blocked requests but does not log what was being sent

---

**Canonical for:** sentry blocker behavior, telemetry blocking strategy, network API interception
*Sentry blocker plugin reference. Last updated 2026-02-19.*
