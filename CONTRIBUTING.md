# Contributing to Uprooted

Guidelines for contributing to the Uprooted framework. Read this before opening a pull request.

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
git clone https://github.com/The-Uprooted-Project/uprooted.git
cd uprooted
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
| Root Communications | desktop app | Runtime testing target |

### Clone and Setup

```bash
git clone https://github.com/The-Uprooted-Project/uprooted.git
cd uprooted
git checkout contrib
pnpm install
```

### Building

```bash
# TypeScript bundle (output goes to dist/)
pnpm build
```

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
fix: theme colors not updating on preset switch
feat: add bridge proxy for voice channel controls
docs: add plugin API reference with lifecycle examples
chore: pin TypeScript to exact version in package.json
```

---

## Pull Request Guidelines

1. Push your changes to the `contrib` branch or a feature branch off `contrib`.
2. Open a Pull Request targeting `main`.
3. Write a clear title and description explaining what changed and why.
4. Ensure the TypeScript build passes (`pnpm build`).
5. If you added code, add or update types accordingly.
6. Link any related GitHub issues.
7. Wait for @watchthelight to review and approve.

Keep PRs focused. One logical change per PR is easier to review than a grab-bag of unrelated modifications.

---

## Reporting Bugs

Use the [bug report template](https://github.com/The-Uprooted-Project/uprooted/issues/new?template=bug-report.yml) on GitHub.

Include:
- Steps to reproduce
- Expected vs. actual behavior
- Root version and OS
- Relevant log output

## Suggesting Features

Use the [feature request template](https://github.com/The-Uprooted-Project/uprooted/issues/new?template=feature-request.yml) on GitHub.

Include:
- Description of the desired behavior
- Why it would be useful
- Any technical considerations

---

## License

By contributing, you agree that your contributions will be licensed under the [Uprooted License v1.0](LICENSE).
