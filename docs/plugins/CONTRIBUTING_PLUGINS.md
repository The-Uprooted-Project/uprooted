# Contributing a Plugin to Uprooted

A step-by-step guide for contributing your plugin to the Uprooted project. Whether you've built something small and useful or an ambitious new feature, this guide walks you through the entire process -- from forking the repo to opening your pull request.

> **Related docs:** [Getting Started](GETTING_STARTED.md) | [API Reference](API_REFERENCE.md) | [Examples](EXAMPLES.md) | [Advanced Development](ADVANCED_DEVELOPMENT.md) | [Root Environment](ROOT_ENVIRONMENT.md) | [Build Guide](../install/BUILD.md) | [Contributing](../../CONTRIBUTING.md) | [Contributing Technical](../dev/CONTRIBUTING_TECHNICAL.md)

## Table of Contents

- [Welcome](#welcome)
- [Before You Start](#before-you-start)
- [Fork and Clone](#fork-and-clone)
- [Set Up Your Development Environment](#set-up-your-development-environment)
- [Create Your Plugin](#create-your-plugin)
- [Build and Test Locally](#build-and-test-locally)
- [Polish Your Plugin](#polish-your-plugin)
- [Prepare and Open Your PR](#prepare-and-open-your-pr)
- [What Reviewers Look For](#what-reviewers-look-for)
- [Common Mistakes](#common-mistakes)
- [Getting Help](#getting-help)
- [Further Reading](#further-reading)

---

## Welcome

Thanks for considering contributing a plugin to Uprooted. Community plugins are what make a mod framework worth using, and we genuinely appreciate every contribution -- from a five-line CSS tweak to a full-featured UI overhaul.

This guide is for contributors who want to **submit a plugin to the main Uprooted repository** so it ships as a built-in plugin for all users. If you just want to build a plugin for personal use, the [Getting Started](GETTING_STARTED.md) tutorial is all you need.

### What to expect

1. You'll fork the repo, create your plugin, and test it locally.
2. You'll open a pull request from a branch off `contrib`.
3. A maintainer will review your code, suggest changes if needed, and merge when it's ready.
4. Your plugin ships in the next Uprooted release.

The review process is collaborative, not adversarial. We will work with you to get your plugin merged.

### What makes a good contribution

The best plugin contributions share a few traits:

- **Solve a real problem.** Does your plugin do something users actually want? A plugin that blocks annoying notifications, improves the call UI, or adds a missing feature is more likely to be accepted than one that exists purely as an experiment.
- **Work reliably.** It doesn't need to be perfect, but it should start cleanly, stop cleanly, and not crash Root.
- **Play well with others.** It shouldn't break existing plugins or Root's own functionality.
- **Clean up after itself.** When disabled, it should leave no trace -- no leftover DOM elements, no orphaned observers, no stale CSS.

---

## Before You Start

### Prerequisites

You'll need these tools installed:

| Tool | Version | Why |
|------|---------|-----|
| Node.js | 18+ | TypeScript build tooling |
| pnpm | 8+ | Package manager (`npm install -g pnpm`) |
| Git | any recent | Version control |
| Root Communications | desktop app | Testing your plugin |
| Uprooted | installed | Runtime environment for your plugin |

You do **not** need the .NET SDK or Rust toolchain unless you're also modifying the hook or installer. Plugin development is TypeScript only.

### Skills assumed

This guide assumes you're comfortable with:

- TypeScript and basic DOM manipulation
- Git workflows (fork, branch, commit, push, pull request)
- Reading existing code to understand patterns

If you haven't built a plugin before, work through the [Getting Started](GETTING_STARTED.md) tutorials first. This guide picks up where that one leaves off.

---

## Fork and Clone

### Step 1: Fork the repository

Go to [github.com/watchthelight/uprooted](https://github.com/watchthelight/uprooted) and click **Fork** in the top-right corner. This creates your own copy of the repository under your GitHub account.

### Step 2: Clone your fork

```bash
git clone https://github.com/YOUR_USERNAME/uprooted.git
cd uprooted
```

### Step 3: Set up the upstream remote

Add the original repo as an upstream remote so you can pull in changes later:

```bash
git remote add upstream https://github.com/watchthelight/uprooted.git
```

### Step 4: Create a working branch off `contrib`

All contributions must go through the `contrib` branch. Never push directly to `main`.

```bash
git checkout contrib
git pull upstream contrib
git checkout -b plugin/my-plugin-name
```

Use the `plugin/` prefix for your branch name -- it makes the purpose clear at a glance.

---

## Set Up Your Development Environment

### Install dependencies

```bash
pnpm install
```

### Verify the build works

```bash
pnpm build
```

This should produce `dist/uprooted-preload.js` and `dist/uprooted.css` without errors. If something fails here, fix it before writing any code -- you want a clean baseline.

### Verify Uprooted is installed

Launch Root Communications and look for the "UPROOTED" section in the settings sidebar. If it's not there, install Uprooted first using the installer or install scripts.

---

## Create Your Plugin

### Naming conventions

Choose a name that is:

- **Lowercase and hyphenated**: `my-cool-plugin`, not `MyCoolPlugin` or `my_cool_plugin`
- **Descriptive**: `voice-activity-overlay` tells you what it does; `vao` does not
- **Unique**: Don't conflict with built-in names (`sentry-blocker`, `themes`, `settings-panel`, `link-embeds`)

Your plugin name is used as:
- The directory name under `src/plugins/`
- The key in the settings JSON (`plugins.my-cool-plugin.config`)
- The CSS element ID prefix (`uprooted-css-plugin-my-cool-plugin`)
- The identifier in log messages

### File structure

Create a directory for your plugin:

```
src/plugins/my-cool-plugin/
├── index.ts       # Main plugin export (required)
├── styles.ts      # CSS strings (optional, for long stylesheets)
└── utils.ts       # Helper functions (optional)
```

The only required file is `index.ts`, which must export a default object satisfying `UprootedPlugin`.

### Plugin template

Start with this template and fill in the blanks:

```typescript
import type { UprootedPlugin } from "../../types/plugin.js";
import { nativeLog } from "../../api/native.js";

export default {
  name: "my-cool-plugin",
  description: "One sentence describing what this plugin does",
  version: "0.1.0",
  authors: [{ name: "YourGitHubUsername" }],

  // Optional: static CSS injected while the plugin is active
  // css: ``,

  // Optional: user-configurable settings
  // settings: {},

  // Optional: bridge method intercepts
  // patches: [],

  start() {
    nativeLog("[my-cool-plugin] Started");
    // Your initialization code here
  },

  stop() {
    nativeLog("[my-cool-plugin] Stopped");
    // Clean up everything you created in start()
  },
} satisfies UprootedPlugin;
```

See [API Reference -- UprootedPlugin Interface](API_REFERENCE.md#uprootedplugin-interface) for the full contract.

### Register your plugin

Open `src/core/preload.ts` and add your plugin after the existing registrations:

```typescript
import myCoolPlugin from "../plugins/my-cool-plugin/index.js";

// Inside main(), after the existing loader.register() calls:
loader.register(myCoolPlugin);
```

Plugins start in registration order, so add yours after the built-in plugins. This ensures your patches run after the built-ins' patches.

---

## Build and Test Locally

### Build

```bash
pnpm build
```

This bundles all plugins (including yours) into `dist/uprooted-preload.js`.

### Deploy

Copy the built bundle to Root's installation directory. If you have the install script:

```bash
powershell -File Install-Uprooted.ps1
```

On Linux:

```bash
bash install-uprooted-linux.sh
```

### Test

1. Launch Root Communications.
2. Join a voice channel (required for bridge objects to become available).
3. Open Root's settings and look for the "UPROOTED" section.
4. Verify your plugin appears in the plugins list.
5. Toggle it on/off and verify it starts and stops cleanly.

### Debugging without DevTools

Root does not expose Chrome DevTools. You have three debugging strategies:

**nativeLog** -- Send messages to Root's .NET logs:
```typescript
import { nativeLog } from "../../api/native.js";
nativeLog("Debug: reached checkpoint A");
```

**DOM indicators** -- Create visible elements to show state:
```typescript
const debug = document.createElement("pre");
debug.id = "my-plugin-debug";
debug.style.cssText = "position:fixed;top:0;left:0;z-index:999999;" +
  "padding:8px;background:#000c;color:#0f0;font:11px monospace;max-height:200px;overflow:auto;";
document.body.appendChild(debug);
debug.textContent += "Event fired\n";
```

**Error banner** -- Catch and display errors visually:
```typescript
start() {
  try {
    // your code
  } catch (err) {
    const banner = document.createElement("div");
    banner.style.cssText = "position:fixed;top:0;left:0;right:0;z-index:999999;" +
      "padding:8px 16px;background:#dc2626;color:#fff;font:12px monospace;";
    banner.textContent = `[my-cool-plugin] Error: ${err}`;
    document.body.appendChild(banner);
  }
}
```

For more debugging strategies, see [Root Environment -- Debugging Strategies](ROOT_ENVIRONMENT.md#debugging-strategies).

### Test checklist

Run through these scenarios before opening your PR:

| Scenario | How to test | What to verify |
|----------|-------------|----------------|
| Clean start | Enable plugin, restart Root | No error banner, startup log message appears |
| Clean stop | Disable plugin, restart Root | No leftover DOM elements, no console errors |
| Re-enable | Toggle off then on again | Plugin works correctly after re-enable |
| Bridge interception | Join voice, trigger your patched methods | `nativeLog` shows expected output |
| DOM re-injection | Navigate away from and back to the call view | Injected elements reappear if applicable |
| Settings | Change settings, restart Root | Behavior reflects new values |
| Error handling | Break a dependency (change a selector) | Graceful degradation, no crash |

---

## Polish Your Plugin

Before opening your PR, take a pass through your code with these quality guidelines in mind.

### Code quality checklist

- [ ] **All resources cleaned up in `stop()`.** Every DOM element, `MutationObserver`, `setInterval`, `setTimeout`, and event listener created in `start()` must be removed in `stop()`. This is the single most common issue in plugin reviews.

- [ ] **Patch handlers wrapped in try/catch.** Errors in `before` or `replace` handlers propagate to Root's own code and can cause unexpected behavior. Always catch and log.

  ```typescript
  before(args) {
    try {
      // your logic
    } catch (err) {
      console.error("[my-cool-plugin] handler error:", err);
    }
  }
  ```

- [ ] **No hardcoded colors.** Use Root's CSS variables for theme compatibility:
  ```css
  background: var(--color-background-secondary, #121A26);
  color: var(--color-text-primary, #F2F2F2);
  ```

- [ ] **Element IDs use your plugin name.** Prefix all injected element IDs with your plugin name to avoid collisions: `my-cool-plugin-overlay`, `my-cool-plugin-badge`.

- [ ] **CSS selectors are scoped.** Don't use generic selectors like `.container` or `div > span`. Scope to your own elements or use plugin-specific data attributes.

- [ ] **No `localStorage` or `IndexedDB`.** Root runs in `--incognito` mode. These APIs are unavailable. Use plugin settings for persistence.

- [ ] **Settings have defaults and descriptions.** Every setting field needs a `default` value and a `description` string. Users should understand what each setting does without reading your source code.

- [ ] **Logging is proportionate.** Use `nativeLog` for startup, shutdown, and errors. Don't log on every bridge event or DOM mutation -- it floods the log file.

### Performance considerations

- **Cache DOM references** instead of querying on every event.
- **Debounce high-frequency handlers.** Methods like `setSpeaking` and `receiveRawPacket` fire many times per second. Don't do expensive work on every call.
- **Use `requestAnimationFrame`** for visual updates triggered by rapid events.
- **Keep `MutationObserver` callbacks lightweight.** Check for your element's existence before creating new nodes.

See [Advanced Development -- Performance Best Practices](ADVANCED_DEVELOPMENT.md#performance-best-practices) for more detail.

### Settings best practices

If your plugin has configurable behavior, define a `settings` object with typed fields:

```typescript
settings: {
  showOverlay: {
    type: "boolean",
    default: true,
    description: "Show the status overlay in the corner",
  },
  overlayPosition: {
    type: "select",
    default: "bottom-right",
    description: "Where to place the overlay",
    options: ["top-left", "top-right", "bottom-left", "bottom-right"],
  },
},
```

Always read settings with fallback defaults in `start()`:

```typescript
const config = window.__UPROOTED_SETTINGS__?.plugins?.["my-cool-plugin"]?.config;
const showOverlay = (config?.showOverlay as boolean) ?? true;
```

This ensures your plugin works even if the user hasn't configured it yet. See [API Reference -- Settings Definition](API_REFERENCE.md#settings-definition) for all four field types.

---

## Prepare and Open Your PR

### Step 1: Sync with upstream

Before opening your PR, make sure your branch is up to date:

```bash
git fetch upstream
git rebase upstream/contrib
```

Resolve any conflicts that come up. Don't discard upstream changes -- merge yours alongside them.

### Step 2: Review your own diff

```bash
git diff upstream/contrib...HEAD
```

Read through every line. Ask yourself:

- Did I leave any debug code in? (`console.log("HERE")`, hardcoded test values)
- Are there any files I changed by accident?
- Does every import use `.js` extensions? (Required for ES module resolution)

### Step 3: Commit with a clear message

Follow the project's commit format:

```
feat: add my-cool-plugin for [what it does]
```

If your plugin is multiple commits, that's fine -- keep each commit focused on one logical change. See [Contributing -- Commit Format](../../CONTRIBUTING.md#commit-format) for the full format guide.

### Step 4: Push your branch

```bash
git push origin plugin/my-cool-plugin
```

### Step 5: Open the pull request

Go to the original repository on GitHub and open a PR from your branch targeting `main`.

Use this template for your PR description:

```markdown
## Summary

[1-3 sentences: what does this plugin do and why is it useful?]

## Plugin details

- **Name:** `my-cool-plugin`
- **Files added:**
  - `src/plugins/my-cool-plugin/index.ts`
  - (any other files)
- **Files modified:**
  - `src/core/preload.ts` (registration)
- **Settings:** [yes/no -- if yes, list them briefly]
- **Bridge patches:** [yes/no -- if yes, list which methods]

## Testing done

- [ ] Starts and stops cleanly
- [ ] No leftover DOM elements after stop
- [ ] Settings work correctly (if applicable)
- [ ] Bridge patches fire as expected (if applicable)
- [ ] Tested with Root v[version]

## Screenshots

[If your plugin has a visual component, include a screenshot or describe what the user sees]
```

### What to include in your PR

Your PR should contain:

1. **Your plugin files** (`src/plugins/my-cool-plugin/`)
2. **The registration line** in `src/core/preload.ts`
3. **Nothing else** unless your plugin requires changes to shared code (which should be discussed first)

Your PR should **not** contain:

- Changes to other plugins
- Changes to the build configuration
- Unrelated formatting or style fixes
- Changes to documentation (unless your plugin introduces a new API pattern worth documenting)

---

## What Reviewers Look For

When a maintainer reviews your PR, they're checking for these things (roughly in order of importance):

### 1. Safety

Does your plugin risk breaking Root or other plugins?

- Patch handlers must not throw unhandled errors (wrap in try/catch)
- `replace` handlers must be used carefully -- they block all other handlers for that method
- DOM manipulation must not interfere with Root's own UI elements
- No calls to dangerous APIs (no `eval`, no dynamic `import()`, no `document.write`)

### 2. Cleanup

Does `stop()` undo everything `start()` did?

- Every `document.createElement` → matched by `.remove()` in stop
- Every `observe()` → matched by calling the disconnect function in stop
- Every `addEventListener` → matched by `removeEventListener` in stop
- Every `setInterval` / `setTimeout` → matched by `clearInterval` / `clearTimeout` in stop

### 3. Correctness

Does the plugin actually work as described?

- Settings are read with proper type assertions and fallback defaults
- Bridge patches target the correct methods with correct argument handling
- CSS uses the proper override pattern (`--rootsdk-*` for theme overrides)
- DOM queries use stable selectors (not obfuscated class names)

### 4. Code quality

Is the code readable and maintainable?

- Clear variable names
- No unnecessary complexity
- TypeScript types used properly (no `any` unless truly necessary)
- Consistent with the rest of the codebase (2-space indent, semicolons, `.js` import extensions)

### 5. User experience

Is the plugin pleasant to use?

- Settings descriptions are clear and helpful
- Visual elements match Root's aesthetic (use CSS variables)
- No performance impact on normal usage
- Graceful behavior when dependencies are missing

---

## Common Mistakes

These are the issues we see most often in plugin PRs. Check your code against this list before submitting.

| Mistake | Why it's a problem | Fix |
|---------|-------------------|-----|
| Missing `stop()` cleanup | Leaked DOM nodes, observers, and listeners accumulate across enable/disable cycles | Match every resource creation in `start()` with cleanup in `stop()` |
| Unguarded `await` in `start()` | `waitForElement` rejection kills the plugin silently | Wrap async `start()` in try/catch, log errors with `nativeLog` |
| Throwing in patch handlers | Error propagates to Root's own code, can cause unexpected behavior | Always wrap handler logic in try/catch |
| Using `localStorage` | Returns `null` or throws in incognito mode | Use plugin settings for persistence |
| Hardcoded colors | Plugin looks wrong when the user switches themes | Use `var(--color-*)` CSS variables with fallback values |
| Generic element IDs | Collides with other plugins or Root's own elements | Prefix all IDs with your plugin name |
| Missing `.js` extension on imports | Build fails with module resolution errors | Always use `import ... from "../../api/native.js"` (with `.js`) |
| Excessive logging | Floods the log file, makes debugging harder for everyone | Log startup, shutdown, and errors only |
| Mutating `args` unintentionally | Changes what Root's own handler receives | Only mutate args when that's your explicit intent; otherwise read-only |
| Not reading settings with fallbacks | Plugin crashes on first run before user configures it | Always `(config?.key as Type) ?? defaultValue` |

---

## Getting Help

If you get stuck at any point:

- **Read the existing plugins.** The built-in plugins in `src/plugins/` are the best examples of how things should work. `themes/` and `sentry-blocker/` cover most common patterns.
- **Check the examples.** [EXAMPLES.md](EXAMPLES.md) has 11 annotated example plugins covering beginner through advanced patterns.
- **Open a draft PR.** If you're unsure about your approach, open a draft PR early. Maintainers can give feedback on your direction before you invest a lot of time.
- **Ask in GitHub Issues.** If you have a question about the API or a technical problem, open an issue. There are no dumb questions.

---

## Further Reading

These documents cover specific aspects of plugin development in more depth:

| Document | What it covers |
|----------|---------------|
| [Getting Started](GETTING_STARTED.md) | First plugin tutorial, project setup, build workflow |
| [API Reference](API_REFERENCE.md) | Full plugin API: interfaces, lifecycle, CSS/DOM/Native/Bridge APIs |
| [Bridge Reference](BRIDGE_REFERENCE.md) | All 71 bridge methods with type signatures |
| [Root Environment](ROOT_ENVIRONMENT.md) | Runtime context: Chromium constraints, CSS variables, debugging |
| [Examples](EXAMPLES.md) | 11 annotated example plugins (beginner to advanced) |
| [Advanced Development](ADVANCED_DEVELOPMENT.md) | Bridge chains, multi-plugin interaction, performance, error recovery |
| [Build Guide](../install/BUILD.md) | Build pipeline for all layers |
| [Contributing](../../CONTRIBUTING.md) | Branch rules, commit format, PR process, code style |
| [Contributing Technical](../dev/CONTRIBUTING_TECHNICAL.md) | Dev environment setup, debugging workflows, failure modes |

---

*Last updated: 2026-02-16*
