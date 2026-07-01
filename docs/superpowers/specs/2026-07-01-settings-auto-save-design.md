# Settings Auto-Save & Vault Folder Picker Design

**Date:** 2026-07-01  
**Status:** Approved

## Summary

Remove the Save button from the Settings window. Every setting change persists immediately via the existing `save_config` Tauri command. Replace the editable vault path text field with a read-only path display and a **Change…** folder picker button.

## Goals

- Eliminate explicit save action — settings apply as the user changes them
- Improve vault selection UX with a folder picker only (no manual path typing)
- Preserve clear feedback via the existing status line

## Non-Goals

- Backend/Rust changes (`get_config`, `save_config`, `pick_vault_folder` stay unchanged)
- Per-field save indicators
- Debounced or partial saves

## UI Changes

### Remove Save button

- Delete the Save button and form submit handler
- Keep the status line (`#status`) at the bottom of the settings panel for success and error messages

### Vault section

| Element | Behavior |
|---------|----------|
| **Use Obsidian default** checkbox | Unchanged; when checked, vault path is `null` |
| **Path display** | Read-only field showing the chosen folder path, truncated with CSS `text-overflow: ellipsis` when long |
| **Placeholder** | When "Use Obsidian default" is checked, show *Using Obsidian default* (or empty with placeholder text) |
| **Change…** button | Opens native folder picker (replaces Browse); disabled when "Use Obsidian default" is checked |
| **Text input** | Removed — no manual path entry |

## Auto-Save Triggers

| Control | Event | Action |
|---------|-------|--------|
| Use Obsidian default | `change` | Save immediately |
| Change… (folder picker) | After folder selected | Uncheck "Use Obsidian default", set path, save immediately |
| Shortcut dropdowns (×3) | `change` | Validate; save if valid |
| Annotation prompt | `change` | Save immediately |
| Text format | `change` | Save immediately |

Shortcut preview updates on `change` (unchanged).

## Validation & Error Handling

### Invalid shortcut

When modifier combination is invalid (duplicate modifiers, or no modifier selected):

- Do **not** call `save_config`
- Show error in status line (existing messages: "Choose different modifiers." / "Pick at least one modifier for the shortcut.")
- Leave invalid selection visible in the form
- App continues using the last successfully saved shortcut

### Save failure

Show `Failed to save: {error}` in status line (unchanged).

### Success

Show **Saved** in status line. Replaced on the next change attempt.

## Architecture

**Approach:** Event-driven save on each control (Approach 1 from brainstorming).

Each control's `change` handler calls the existing `saveConfig()` function. No debouncing, no partial saves. Config is small and saves are fast.

```
User changes control
  → saveConfig()
    → validate shortcut (if shortcut fields involved)
    → invoke("save_config", { config })
    → applyConfig(config) on success
    → setStatus("Saved") or error
```

No Rust changes required.

## Files to Modify

| File | Changes |
|------|---------|
| `index.html` | Vault UI (read-only path + Change…), remove Save button |
| `src/main.ts` | Auto-save listeners, vault display logic, remove form submit |
| `src/styles.css` | Read-only path display styles, remove `#save-btn` styles |
| `docs/screenshots/settings-mockup.html` | Sync mockup with new UI |

## Testing

Manual verification:

1. Open Settings from tray — all fields load correctly
2. Toggle "Use Obsidian default" — saves immediately, status shows "Saved", Change… disabled
3. Pick folder via Change… — path displays, saves immediately, checkbox unchecked
4. Change text format / annotation prompt — saves on change
5. Set invalid shortcut — error shown, previous shortcut still active (verify clip still works)
6. Set valid shortcut — saves and rebinds (verify new shortcut works)
7. Dark mode — path display readable
