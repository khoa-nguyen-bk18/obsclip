# Obsclip Design Spec

**Date:** 2026-06-29  
**Status:** Approved  
**Platforms (v1):** macOS, Windows  
**Platforms (deferred):** Linux (v1.1 — architecture kept portable)

## Summary

Obsclip is a small Tauri tray utility that reads Obsidian vault configuration from disk and appends the current clipboard contents (text or image) to today's daily note — without launching Obsidian.

## Goals

- One-action clip from clipboard → today's daily note
- Support text and image clipboard content
- Images saved to the vault's configured attachment folder automatically
- Operate entirely via filesystem (no Obsidian URI, no Obsidian process)
- Global keyboard shortcut + system tray/menu bar
- Minimal settings window accessible from tray

## Non-Goals (v1)

- Linux support (deferred to v1.1; platform paths abstracted for future addition)
- Templater / community plugin template syntax
- Custom image naming patterns
- Append-to-specific-section within a note
- Clip history or undo
- Rich notification toasts (tray flash only)

## User Requirements

| Requirement | Decision |
|-------------|----------|
| Trigger | Global shortcut **and** tray menu action |
| Vault selection | Auto: last-open vault from `obsidian.json`; optional manual override in settings |
| Text format (default) | Timestamped entry: `- HH:mm — {text}` |
| Text format (configurable) | Blockquote, bullet, checkbox |
| Image storage | `attachmentFolderPath` from `.obsidian/app.json`; filename `clip-YYYY-MM-DD-HHmmss.png` |
| Image link in note | Timestamped wikilink: `- HH:mm — ![[clip-....png]]` |
| Missing daily note | Create it (from template if configured), then append |
| Feedback | Tray icon flashes ✓ (success) or ✗ (error) for ~1.5s |
| Settings (v1) | Vault folder picker, shortcut rebinding, text format picker |

## Architecture

```
┌─────────────────────────────────────────────────┐
│  Tauri shell (tray + settings window)           │
│  ┌─────────────┐  ┌──────────────────────────┐  │
│  │ Tray icon   │  │ Settings (vault, shortcut,│  │
│  │ + flash     │  │  text format)            │  │
│  └──────┬──────┘  └──────────────────────────┘  │
│         │ global shortcut                        │
│         ▼                                        │
│  ┌─────────────────────────────────────────────┐│
│  │ Rust: ClipService                           ││
│  │  1. Resolve vault                           ││
│  │  2. Read clipboard (text | image)           ││
│  │  3. Resolve/create daily note               ││
│  │  4. Save image / format text / append       ││
│  └─────────────────────────────────────────────┘│
└─────────────────────────────────────────────────┘
         │ reads                    │ writes
         ▼                          ▼
  obsidian.json              vault/**/*.md
  .obsidian/daily-notes.json attachments/
  .obsidian/app.json
```

### Components

| Component | Responsibility |
|-----------|----------------|
| `VaultResolver` | Resolve vault path (settings override → `obsidian.json` `last_open`) |
| `ObsidianConfig` | Parse `.obsidian/daily-notes.json`, `.obsidian/app.json` |
| `DailyNotePath` | Format today's date per Obsidian format string; build file path |
| `DailyNoteCreator` | Create note from template with core variable substitution |
| `ClipboardReader` | Read text or image from OS clipboard |
| `TextFormatter` | Apply timestamped / blockquote / bullet / checkbox formatting |
| `ImageWriter` | Save PNG to attachment folder; return wikilink filename |
| `ClipService` | Orchestrate clip flow; return success/error |
| `AppConfig` | Persist user settings (`config.json`) |
| `TrayController` | Tray menu, flash feedback, open settings |
| `ShortcutController` | Register/rebind global shortcut |

### Platform Paths

| OS | Obsidian global config | Obsclip config |
|----|------------------------|----------------|
| macOS | `~/Library/Application Support/obsidian/obsidian.json` | `~/Library/Application Support/obsclip/config.json` |
| Windows | `%APPDATA%\obsidian\obsidian.json` | `%APPDATA%\obsclip\config.json` |
| Linux (v1.1) | `$XDG_CONFIG_HOME/obsidian/obsidian.json` | `$XDG_CONFIG_HOME/obsclip/config.json` |

Platform path resolution lives in a single `platform.rs` module to ease Linux addition later.

## Data Flow: Clip

1. User triggers clip (shortcut or tray).
2. `VaultResolver` returns vault root path.
3. `ObsidianConfig` loads daily-notes and app settings.
4. `ClipboardReader` detects content type:
   - **Text:** `TextFormatter` formats per setting.
   - **Image:** `ImageWriter` saves to `{vault}/{attachmentFolderPath}/clip-{timestamp}.png`.
5. `DailyNotePath` computes `{folder}/{formatted-date}.md`.
6. If note missing: `DailyNoteCreator` creates from template (if set).
7. Append formatted content with `\n\n` separator.
8. `TrayController` flashes success or error.

## Obsidian Config Reference

### `obsidian.json` (global)

```json
{
  "vaults": {
    "96a832d9c9cc9eca": {
      "path": "/Users/me/vault",
      "ts": 1643208916609,
      "open": true
    }
  },
  "last_open": "96a832d9c9cc9eca"
}
```

Resolution order: settings `vault_path` override → `vaults[last_open].path` → error.

### `.obsidian/daily-notes.json`

```json
{
  "format": "YYYY-MM-DD",
  "folder": "Daily",
  "template": "Templates/Daily Note"
}
```

- `format`: Obsidian/moment.js date tokens (`YYYY`, `MM`, `DD`, `MMMM`, etc.)
- `folder`: subfolder relative to vault root (may be empty = vault root)
- `template`: note path without `.md` extension (optional)

### `.obsidian/app.json`

```json
{
  "attachmentFolderPath": "./attachments"
}
```

Default to `attachments/` if key missing. Strip leading `./`.

## Text Formatting

| Format | Single-line input | Multi-line input |
|--------|-------------------|------------------|
| Timestamped (default) | `- HH:mm — line` | First line timestamped; continuation lines indented 2 spaces |
| Blockquote | `> line` | `> line` per line |
| Bullet | `- line` | `- line` per line |
| Checkbox | `- [ ] line` | `- [ ] line` per line |

Image entries always use timestamped wikilink format regardless of text format setting.

## Template Substitution (v1)

When creating a new daily note, if `template` is set, copy template file content and substitute:

| Variable | Replacement |
|----------|-------------|
| `{{title}}` | Daily note filename without extension |
| `{{date}}` | Today's date in daily note format |
| `{{time}}` | Current time `HH:mm` |

Obsidian's `{{date:FORMAT}}` variant: substitute inner format via same date formatter.

Templater syntax (`<% ... %>`) is copied verbatim — not evaluated.

## UI

### Tray Menu

- **Clip to daily note** — runs clip flow
- **Settings…** — opens settings window
- **Quit**

### Tray Feedback

- Success: overlay/badge ✓ (green tint), 1.5s
- Error: overlay/badge ✗ (red tint), 1.5s
- No OS toast notifications in v1

### Settings Window

| Field | Type | Default |
|-------|------|---------|
| Vault folder | Folder picker + "Use Obsidian default" | Auto |
| Global shortcut | Key recorder | `Cmd+Shift+V` / `Ctrl+Shift+V` |
| Text format | Select | Timestamped |

App launches to tray only — no main window on startup.

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Clipboard empty / unsupported | Tray flash ✗ |
| Vault not resolvable | Tray flash ✗; settings shows hint |
| Vault path not writable | Tray flash ✗ |
| `daily-notes.json` missing | Tray flash ✗ (daily notes plugin not configured) |
| Template file missing | Create note with `# {date}` heading fallback |
| Image save fails | Tray flash ✗ |

## Tech Stack

- **Tauri 2.x** — tray, window, IPC
- **Rust:** `arboard` (clipboard), `chrono` (dates), `serde`/`serde_json`, `tauri-plugin-global-shortcut`
- **Frontend:** minimal HTML/CSS/JS for settings panel only
- **Tests:** Rust unit tests for config parsing, date formatting, text formatting, path resolution

## Testing Strategy

- Unit tests with fixture JSON/files for vault resolution, daily note path, formatters
- Manual integration: clip text, clip image, missing daily note creation, settings persistence
- Manual on macOS and Windows before release

## Future (v1.1+)

- Linux support (tray + shortcuts on X11/Wayland)
- Additional text formats / custom timestamp pattern
- Custom image filename pattern
- Optional OS notifications
