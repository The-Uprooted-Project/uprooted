# Message Logger

Logs deleted and edited messages. Deleted messages appear with red styling; edited messages show the previous content with an amber edit indicator.

**Status:** Beta
**Layer:** Native (Avalonia)

---

## What it does

When someone deletes a message, Root removes it from the chat entirely. Message Logger preserves these messages and displays them with visual indicators so you can see what was removed.

| Event | Default Root behavior | With Message Logger |
|-------|----------------------|-------------------|
| Message deleted | Message disappears from chat | Message stays, shown with red full-width stripe and 3px red left border |
| Message edited | Content silently replaced | Previous content shown with amber left border + "(edited)" label |

## Storage

Logged messages are stored locally in a flat file:

- **Location:** `{Root profile dir}/uprooted-message-log.dat`
- **Format:** pipe-delimited append-only records
- **Retention:** configurable max message count
- **Privacy:** local storage only, no network sync

## Settings

| Setting | Default | Description |
|---------|---------|-------------|
| Log Deleted Messages | On | Enable deletion detection |
| Log Edited Messages | On | Enable edit detection and amber indicators |
| Ignore Own Messages | Off | Skip logging your own messages |
| Max Messages | 500 | Retention limit |

## Known Limitations

- **Edit detection needs validation**: the grace period filter may occasionally produce false positives with rapid message updates
- **No image/attachment logging**: only text content is logged
- **Storage is local-only**: no sync across devices
