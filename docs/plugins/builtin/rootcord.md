# Rootcord

Transforms Root's horizontal tab bar into a Discord-style vertical server strip on the left side of the window.

**Status:** Experimental
**Layer:** Native (Avalonia)

---

## What it does

| Feature | Without Rootcord | With Rootcord |
|---------|-----------------|---------------|
| Navigation | Horizontal tab bar across the top | Vertical server strip on the left (56px wide) |
| Server switching | Click tabs in top bar | Click server icons in left strip |
| Community members sidebar | Left side of chat | Right side of chat |
| Member profile flyouts | Open to the right | Open to the left |
| Utility pane (friends, DMs, notifications) | Opens on the right | Opens on the left |
| User info | In Root's native header | Floating user card at bottom-left |

## Server strip

The server strip is a 56px-wide panel containing:

- **Home button**: envelope icon that opens your DMs
- **Server icons**: one per community, with the server's logo or first-letter fallback
- **Selection pill**: white rounded indicator on the active server
- **Unread badges**: red dot for mentions, orange for unread
- **Tooltips**: server name and member count on hover

## User bar

A floating card at the bottom-left showing:

- Avatar and display name
- Online status
- Quick-action buttons: Friends, DMs, Notifications, Settings

## Live toggle

Rootcord can be toggled on and off from Plugin Settings without restarting Root. Disabling it cleanly restores the original horizontal tab layout.

## Settings

| Setting | Default | Description |
|---------|---------|-------------|
| Enable Rootcord | Off | Toggle the vertical server sidebar |
| Use Original Server Bar | Off | Use Root's native tab bar styling instead of Rootcord's custom strip |

## Known Limitations

- **Right gutter in community view**: a small gap may appear on the right edge of the members panel in some layouts
- **Flyout positioning**: member profile popups may briefly appear on the wrong side before being repositioned
- **Username fallback**: "User" placeholder shown briefly on first startup before the session loads
- **Profile picture**: avatar shows first letter if the profile picture hasn't loaded yet
