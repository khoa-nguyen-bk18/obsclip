# Obsclip

Obsclip is a small system-tray utility that appends your current clipboard (text or image) to today's Obsidian daily note. It reads your vault configuration from disk and writes files directly—no need to open Obsidian or use Obsidian URI schemes.

## Requirements

- **Obsidian** with **Daily notes** enabled (and a daily-notes folder configured as you normally would in Obsidian)
- **macOS** or **Windows**

> **Linux** is planned for v1.1. The codebase is kept portable, but Linux is not supported in v1.

## Install and development

```bash
npm install
npm run tauri dev
```

To build the frontend assets only:

```bash
npm run build
```

## Global shortcut

Default shortcut:

- **macOS:** `Cmd+Shift+V`
- **Windows:** `Ctrl+Shift+V`

You can change the shortcut in Settings (see below).

## System tray

Obsclip runs in the menu bar (macOS) or system tray (Windows). Click the tray icon to open the menu:

- **Clip to daily note** — Appends the current clipboard contents to today's daily note (creates the note from your daily template if needed). Text entries use your chosen format; images are saved to the vault attachment folder and linked in the note.
- **Settings…** — Opens the settings window.
- **Quit** — Exits Obsclip.

After a clip, the tray tooltip briefly shows **✓ Clipped** on success or **✗ Error** on failure.

## Settings

Open **Settings…** from the tray menu to configure:

- **Vault** — Optional vault path override, or **Use Obsidian default** to follow the last-open vault from Obsidian's `obsidian.json`.
- **Global shortcut** — Rebind the clip shortcut (Tauri format, e.g. `CommandOrControl+Shift+KeyV`). Save to apply.
- **Text format** — How text clips are written: timestamped (default), blockquote, bullet, or checkbox.

Click **Save** to persist changes.
