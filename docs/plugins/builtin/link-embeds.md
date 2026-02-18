# Link Embeds

Discord-style rich link previews for URLs. Fetches OpenGraph metadata and renders embed cards inline. YouTube URLs get special treatment with thumbnail-to-player embeds.

> **Source:** [`src/plugins/link-embeds/`](../../../src/plugins/link-embeds/) -- `index.ts`, `providers.ts`, `embeds.ts`

---

## What it does

When a URL appears in the page, this plugin:

1. Detects the `<a>` element via MutationObserver
2. Fetches metadata (OpenGraph tags for websites, oEmbed for YouTube)
3. Renders a styled embed card below the link
4. For YouTube links, shows a clickable thumbnail that expands to an inline player

## Settings

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `youtube` | boolean | `true` | Show YouTube video embeds |
| `websites` | boolean | `true` | Show website link previews |
| `maxEmbedsPerMessage` | number | `3` | Maximum embeds per message context (1--10) |

## Embed Types

### Generic Link Embeds

For non-YouTube URLs, the plugin fetches the page HTML and parses OpenGraph meta tags:

```
og:title       -> Embed title (clickable link)
og:description -> Description (max 250 chars, 3-line clamp)
og:image       -> Thumbnail (80x80, right-aligned, removed on 404)
og:site_name   -> Provider name (falls back to hostname)
theme-color    -> Left border accent color
```

The card layout is a flex row: text body on the left, optional thumbnail on the right, with a 4px colored left border.

If no `og:title` is found, falls back to the `<title>` tag. If neither exists, no embed is rendered.

### YouTube Embeds

YouTube URLs are detected by hostname (`youtube.com`, `m.youtube.com`, `youtu.be`) and parsed for video ID from these patterns:
- `/watch?v=ID`
- `/embed/ID`
- `/shorts/ID`
- `youtu.be/ID`

Metadata is fetched via YouTube's oEmbed endpoint (`/oembed?url=...&format=json`), which returns the video title and channel name.

The card layout is a flex column: text body on top, video section below. The video section shows:
- Thumbnail from `img.youtube.com/vi/{id}/hqdefault.jpg`
- Red play button overlay (SVG)
- Click replaces thumbnail with an autoplay iframe (`youtube.com/embed/{id}?autoplay=1`)

## How it works

### Link Detection

- `MutationObserver` watches `document.body` for added nodes with `childList: true, subtree: true`
- Each added element is scanned for `<a href>` elements matching `^https?://`
- A `WeakSet<HTMLAnchorElement>` tracks processed links to prevent duplicates
- Links inside Uprooted's own UI (`[id^="uprooted-"], [data-uprooted]`) are skipped

### Metadata Fetching

- **Cache:** `Map<string, EmbedData | null>` -- caches both successes and failures
- **Timeout:** 5 seconds via `AbortController`
- **Size limit:** Only the first 50KB of HTML is read (via `ReadableStream`) to avoid loading entire pages
- **OG parsing:** Regex-based extraction handles both attribute orders (`property` before `content` and vice versa)
- **Relative images:** Resolved via `new URL(image, pageUrl).href`

### Embed Placement

The plugin walks up from the anchor to find the nearest block-level parent (`display: block|flex|grid`), then inserts the embed card after it. Falls back to inserting directly after the anchor.

### Per-Message Limits

`countEmbedsInContext()` walks 5 parent levels up from the anchor and counts existing `.uprooted-embed` elements in that container. If the count meets `maxEmbedsPerMessage`, no more embeds are added. The limit is re-checked after the async metadata fetch in case other embeds were added concurrently.

### Safety

- All text content uses `document.createTextNode()` (no innerHTML for user data)
- Links use `target="_blank" rel="noopener noreferrer"`
- Images use `loading="lazy"` and are removed on 404 via `img.onerror`

## Lifecycle

**start():**
1. Creates a MutationObserver watching `document.body`
2. Scans all existing `<a>` elements in the current DOM

**stop():**
1. Disconnects the observer
2. Removes all `.uprooted-embed` elements from the DOM
3. Clears the metadata cache

## Known Limitations

- **CORS failures** -- despite `--disable-web-security`, some servers may still reject cross-origin fetches. No embed is rendered on failure.
- **50KB HTML limit** -- pages with OpenGraph tags deep in the HTML (after 50KB) won't have their metadata extracted
- **Chat is Avalonia-native** -- this TypeScript plugin only works in the DotNetBrowser context (WebRTC, sub-apps). Chat link embeds are handled by the C# hook's `LinkEmbedEngine` (Avalonia-native, Phase 4.5b). Supports YouTube, Twitter/X, direct images, embed-fixer domains (vxtwitter, fxtwitter, fixupx), and any site with OpenGraph or oEmbed tags.
- **No embed for title-less pages** -- if a page has no `og:title`, `twitter:title`, and no `<title>`, no embed is created
- **No animated image preview** -- `.gif` and animated `.webp` render as static thumbnails
- **No .mp4 preview** -- direct video URLs may hit Cloudflare challenges or render as "Just a moment..." text embeds
- **Trimming constraints** -- `Regex.Replace` with lambdas and `ReadAsStringAsync` are trimmed in Root's binary; all JSON decoding and HTTP body reading uses manual loops and `ReadAsStreamAsync` instead
- **Verbose logging** -- detailed HTTP step logs available via `UPROOTED_VERBOSE=1` environment variable
