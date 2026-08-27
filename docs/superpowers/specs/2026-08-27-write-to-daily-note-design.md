# Write to Daily Note Design

**Date:** 2026-08-27  
**Status:** Approved  
**Platforms:** macOS, Windows (same as Obsclip v1)

## Summary

Add a second capture path: a global shortcut (default `⌘⇧N` / `Ctrl+Shift+N`) and a tray item **Write to daily note** open a multiline compose dialog. Typed text is appended to today's daily note using the same Text format setting and `run_clip` path as clipboard **text** clips. Clipboard is not read.

This is separate from the existing clip-annotation dialog (optional one-line note attached to a clipboard clip).

## Goals

- Type a note (including multiple lines) and insert it into today's daily note without using the clipboard
- Trigger via configurable global shortcut **and** tray menu
- Reuse existing vault resolve, daily-note create, text formatting, append, and tray flash
- Keep the clipboard clip flow (shortcut, tray **Clip to daily note**, optional annotation prompt) unchanged

## Non-Goals (v1)

- Reusing or morphing the annotation window into compose mode
- Native OS text prompts
- Preview of formatted markdown in the compose dialog
- Buttons in the compose dialog (keyboard-only, matching annotation)
- Clip history, undo, or append-to-section
- Linux (deferred with the rest of Obsclip)
- Changing how clipboard clips or annotation work

## User Requirements

| Requirement | Decision |
|-------------|----------|
| Triggers | Global write shortcut **and** tray **Write to daily note** |
| Default shortcut | `CommandOrControl+Shift+KeyN` (`⌘⇧N` / `Ctrl+Shift+N`) |
| Shortcut settings | Second fieldset in Settings, same three pickers + live preview as clip |
| Body | Multiline textarea; **Enter** = new line; **⌘↵** / **Ctrl+↵** = submit; **Esc** = cancel |
| Formatting | Same as clipboard text clips — current **Text format** setting. Multiline timestamped layout already exists in the formatter |
| Empty / whitespace submit | Close with no write (same as cancel). No tray flash |
| Cancel | Esc or window close: hide, write nothing, no flash |
| Clipboard | Not read |
| Annotation prompt setting | Does not apply; compose always shows a dialog |
| OCR | Unused (text-only path) |
| Overlays | Only one of compose or annotation is open; opening one cancels the other |
| Feedback | Same tray flash as clip (green success, red error). No new toasts |

### Example output (timestamped, two lines)

```markdown
- 11:24 — First line
  Second line
```

## Architecture

Dedicated compose WebView, sibling to the annotation window — not a mode of it.

```
Shortcut / tray "Write to daily note"
        │
        ▼
  handle_write
        │
        ├── cancel pending annotation (if any)
        └── show compose window (empty textarea)
                │
                ├── Esc / close / empty submit → hide, no write
                └── non-empty submit
                        │
                        ▼
                  run_clip(ClipboardContent::Text(trimmed))
                        │
                        ▼
                  tray flash (success / error)
```

### Components

| Component | Responsibility |
|-----------|----------------|
| `AppConfig.write_shortcut` | Persist write shortcut; default `CommandOrControl+Shift+KeyN`; missing field loads that default |
| Compose WebView (`compose.html`) | Hidden always-on-top skip-taskbar window; textarea + hint |
| `ComposeState` | Session id + completed flag so double-submit / close cannot clip twice |
| `handle_write` | Load config; on failure flash red and return; otherwise show compose |
| `submit_compose` / `cancel_compose` | Hide window; submit only calls `run_clip` when trimmed text is non-empty |
| Tray **Write to daily note** | Same `handle_write` as the shortcut |
| Shortcut controller | Register/rebind clip and write shortcuts independently. `save_config` rebinds a shortcut only when its stored value changed (same as today’s clip rebind). Settings rejects an identical pair |

Existing `ClipService` / `run_clip` / `format_text` are unchanged. Compose passes `annotation: None`, `image_ocr` unused because content is text.

### Window

New Tauri window, parallel to `annotation`:

| Property | Value |
|----------|--------|
| Label / URL | `compose` / `compose.html` |
| Size | Slightly taller than annotation (textarea); not resizable |
| `visible` | `false` until shown |
| `alwaysOnTop` | `true` |
| `skipTaskbar` | `true` |

Capability list includes `compose`. Close-requested hides the window and cancels (does not destroy it), same as annotation.

## Data Flow

1. User presses the write shortcut or chooses **Write to daily note**.
2. If the annotation dialog is open, it cancels (pending clipboard clip is abandoned). Compose centers, shows, focuses an empty textarea.
3. **Enter** inserts a newline. **⌘↵** / **Ctrl+↵** submits. **Esc** or close hides with no write.
4. Submit trims. Empty → hide and stop. Non-empty → hide, then `run_clip` with `ClipboardContent::Text`, current `text_format`, `annotation: None`.
5. `run_clip` resolves vault, creates today's note if needed, formats, appends with the existing `\n\n` separator.
6. Tray flashes green or red.

Opening annotation while compose is open cancels compose (no write).

Settings save of `write_shortcut` unregisters the old binding and registers the new one, same as clip. Clip and write shortcuts must not be identical; collision shows an error and that change is not saved. Each shortcut still requires at least one modifier.

## UI

**Compose window**

- Textarea, empty on each open, placeholder `Write to today's note…`
- Hint: `⌘↵ to insert · Esc to cancel` (Windows: `Ctrl+↵`)
- No formatted preview, no OCR line, no buttons

**Settings**

- Keep existing **Global shortcut** fieldset for clipboard clip
- Add **Write shortcut** fieldset: same three pickers + preview (default `⌘⇧N`)
- Auto-save on change, same as other settings

**Tray order**

1. Clip to daily note  
2. Write to daily note  
3. Settings…  
4. Quit  

## Error Handling

| Case | Behavior |
|------|----------|
| Config load fail before show | Flash red; do not show compose |
| `run_clip` fail (no vault, IO, etc.) | Hide compose; flash red |
| Empty/whitespace submit, Esc, close | Hide; no write; no flash |
| Invalid write shortcut (no modifier) | Status error; do not save; previous binding kept |
| Write shortcut equals clip shortcut | Status error; do not save |
| Register fail on save | Status error; previous binding kept |
| Write shortcut taken by another app at startup | Log; leave write unregistered; clip shortcut still registers. User can pick a free shortcut in Settings |
| Compose vs annotation both triggered | Opening one cancels the other |

Missing vault is not checked before showing the dialog. Failure happens on submit via `run_clip`, matching clip-with-annotation.

## Testing

Reuse existing `run_clip` and formatter tests. Typed notes are `ClipboardContent::Text` (multiline timestamped layout already covered).

Add:

- Config default `write_shortcut` is `CommandOrControl+Shift+KeyN`
- Config without `write_shortcut` still loads and gets that default
- Settings `saveConfig`: identical clip and write shortcuts are rejected (same status-line pattern as invalid modifiers)
- Compose payload helper: trim empty → `None`; non-empty → `Some(trimmed)`

No Playwright/E2E for the window in v1 (annotation has none).

Manual: write shortcut, tray item, multiline insert, Esc, empty submit, Settings collision, overlay cancel vs annotation.

## Files (expected)

| Area | Files |
|------|--------|
| Frontend | `compose.html`, `src/compose.ts`, `src/compose.css` |
| Settings | `index.html`, `src/main.ts` — second shortcut pickers |
| Rust | `src-tauri/src/compose.rs`, `config.rs`, `lib.rs`, `tray.rs` |
| Tauri | `tauri.conf.json` window, `capabilities/default.json` |
| Tests | `config.rs` unit tests; small helper test for empty vs submit payload |
| Docs | README tray, shortcut, settings rows |
)
