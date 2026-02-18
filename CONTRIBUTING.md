# Contributing to Uprooted

Guidelines for contributing to the Uprooted framework. Read this before opening a pull request.

> **Related docs:** [Documentation Index](docs/INDEX.md) | [Build Guide](docs/install/BUILD.md) | [Architecture](docs/framework/ARCHITECTURE.md)

---

## Status

Uprooted is in **active development** and accepting contributions across the entire project -- framework code, plugins, documentation, tooling, and bug fixes. See [Branch Rules](#branch-rules) for how to submit your work.

---

## Branch Rules

All contributors must push to the `contrib` branch or a feature branch off `contrib`. Direct pushes to `main` are rejected.

- Clone the repo, check out `contrib`, and push your changes there.
- When your work is ready, open a Pull Request from `contrib` (or a feature branch) into `main`.
- Only @watchthelight can approve and merge PRs into `main`.

```bash
# Standard workflow
git clone https://github.com/watchthelight/uprooted-private.git
cd uprooted-private
git checkout contrib
# make your changes, then:
git push origin contrib
```

For larger features, create a branch off `contrib`:

```bash
git checkout contrib
git checkout -b my-feature
# work on your feature, then:
git push origin my-feature
# open a PR targeting main
```

Always pull before starting work -- another contributor may have pushed changes.

---

## Development Setup

### Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| Node.js | 18+ | TypeScript build tooling |
| pnpm | 8+ | Package manager |
| .NET SDK | 10 | C# hook compilation |
| Rust / Cargo | stable | Tauri installer build |
| Root Communications | desktop app | Runtime testing target |

### Clone and Setup

```bash
git clone https://github.com/watchthelight/uprooted-private.git
cd uprooted-private
git checkout contrib
pnpm install
```

### Building

Each layer builds independently. See [BUILD.md](docs/install/BUILD.md) for full details.

```bash
# TypeScript bundle (from public repo, output goes to dist/)
pnpm build

# C# hook
dotnet build hook/ -c Release

# Tauri installer
cd installer && cargo tauri build

# Full installer with embedded artifacts
powershell -File scripts/build_installer.ps1
```

### Running with Root

1. Build the hook and TypeScript bundle.
2. Run the installer (or the install scripts) to patch Root's HTML and deploy the hook DLL.
3. Launch Root. The profiler attaches automatically via environment variables.
4. Check `uprooted-hook.log` (in Root's app directory) for hook startup diagnostics.
5. Open the browser DevTools console (if accessible) to see TypeScript-side `[Uprooted]` log messages.

---

## Code Style

### TypeScript

- Strict mode enabled (`"strict": true` in tsconfig)
- ES modules with explicit `.js` extensions on all imports
- No default exports except for plugin definition objects
- Use `import type` for type-only imports
- 2-space indentation, semicolons
- Descriptive variable names -- no abbreviations (`pluginSettings`, not `ps`)
- Constants in `UPPER_SNAKE_CASE`; functions and variables in `camelCase`; types and interfaces in `PascalCase`
- All logs prefixed with `[Uprooted]` or `[Uprooted:plugin-name]`

### C#

- Namespace: `Uprooted` (exception: `Entry.cs` uses `UprootedHook`, `StartupHook` is global)
- Access: `internal` for all non-entry classes
- All Avalonia type access through `AvaloniaReflection` -- never `typeof()` or `Type.GetType()`
- `Interlocked.CompareExchange` for one-time initialization guards (never `lock` or `bool` flags)
- Never throw from injected code -- catch and log
- Log messages use `[Category]` prefix
- Thread-safe file logging to `uprooted-hook.log`

### Rust (installer)

- Standard Rust conventions (`cargo fmt`, `cargo clippy`)
- Functions return `Result` types for error propagation
- Use `anyhow` for application-level errors

### General

- No abbreviations in variable or function names
- Comments explain "why", not "what"
- JSDoc on exported TypeScript functions
- Section dividers (`// -- Section Name --`) for long files

---

## Commit Format

Every commit message follows this format:

```
type: concise description of what changed
```

### Types

| Type | Use for |
|------|---------|
| `fix` | Bug fixes |
| `feat` | New features or capabilities |
| `refactor` | Code restructuring without behavior change |
| `docs` | Documentation changes |
| `chore` | Build scripts, CI, tooling, dependency updates |
| `style` | Formatting, whitespace, cosmetic changes |

### Examples

```
fix: self-heal HTML patches after Root auto-update overwrites
feat: add Phase 0 startup verification for HTML patches
refactor: prefer in-place stripping over stale backup restore
docs: add plugin API reference with lifecycle examples
chore: pin TypeScript to exact version in package.json
style: normalize 2-space indentation in ContentPages.cs
```

A message body is optional but encouraged for non-obvious changes. Separate the body from the subject with a blank line:

```
fix: prevent back button freeze when injected controls are in visual tree

PointerPressed subscription on Root's back button now calls
CleanupInjection() before Root's own handler fires. Previously,
injected controls could remain in the visual tree during navigation
teardown, causing a UI freeze.
```

---

## Pull Request Guidelines

1. Push your changes to the `contrib` branch or a feature branch off `contrib`.
2. Open a Pull Request targeting `main`.
3. Write a clear title and description explaining what changed and why.
4. Ensure all components build without errors (`pnpm build`, `dotnet build hook/ -c Release`).
5. If you added code, add or update types accordingly.
6. Link any related GitHub issues.
7. Wait for @watchthelight to review and approve.

Keep PRs focused. One logical change per PR is easier to review than a grab-bag of unrelated modifications.

---

## Critical Rules

These constraints exist because violating them causes real, hard-to-diagnose bugs. See [Architecture](docs/framework/ARCHITECTURE.md) for detailed explanations.

- **Never use `Type.GetType()` for Avalonia types** -- use `AvaloniaReflection`
- **Never modify `ContentControl.Content` directly** -- causes UI freeze; use Grid overlay
- **Never use `System.Text.Json` in the hook** -- causes `MissingMethodException` in profiler context
- **Never use `EventInfo.AddEventHandler` for RoutedEvents** -- use Expression lambdas
- **Never use `localStorage`** -- Root runs Chromium with `--incognito`
- **`DispatcherPriority` is a struct, not an enum** in Avalonia 11+

---

## Reporting Bugs

Use the [bug report template](https://github.com/watchthelight/uprooted/issues/new?template=bug-report.yml) on GitHub.

Include:
- Steps to reproduce
- Expected vs. actual behavior
- Root version and OS
- Relevant log output (`uprooted-hook.log`, browser console)

## Suggesting Features

Use the [feature request template](https://github.com/watchthelight/uprooted/issues/new?template=feature-request.yml) on GitHub.

Include:
- Description of the desired behavior
- Why it would be useful
- Any technical considerations

---

## License

By contributing, you agree that your contributions will be licensed under the [Uprooted License v1.0](LICENSE).
