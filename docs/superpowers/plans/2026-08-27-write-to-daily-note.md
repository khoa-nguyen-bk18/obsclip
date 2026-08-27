# Write to Daily Note Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a configurable global shortcut (default `⌘⇧N` / `Ctrl+Shift+N`) and tray item that open a multiline compose dialog and append typed text to today's daily note using the existing text-clip path.

**Architecture:** Dedicated compose WebView (sibling of the annotation window). Typed text is submitted to existing `run_clip` as `ClipboardContent::Text`. Clip shortcut, annotation dialog, and clipboard flow stay unchanged except: opening compose cancels a pending annotation, and opening annotation cancels compose. Overlay cancel is coordinated in `tray.rs` so `compose` and `annotation` do not import each other.

**Tech Stack:** Tauri 2, Rust, vanilla HTML/CSS/TypeScript (Vite)

**Spec:** `docs/superpowers/specs/2026-08-27-write-to-daily-note-design.md`

## Global Constraints

- Platforms: macOS, Windows (same as Obsclip v1)
- Default write shortcut: `CommandOrControl+Shift+KeyN`
- Typed notes use the current **Text format** setting (same as clipboard text clips)
- Clipboard is not read on this path
- Enter = newline; `⌘↵` / `Ctrl+↵` = submit; Esc / window close = cancel
- Empty or whitespace submit = cancel (hide, no write, no tray flash)
- One overlay: compose **or** annotation, never both
- Feedback: existing tray flash only; no new toasts
- Do not change clipboard clip behavior, annotation one-line UI, or OCR

---

## File Structure

```
compose.html                         # compose window (new)
src/compose.ts                       # textarea + submit/cancel invokes (new)
src/compose.css                      # compose window styles (new)
index.html                           # Write shortcut fieldset
src/main.ts                          # second shortcut pickers + collision check
src-tauri/tauri.conf.json            # compose window; taller settings
src-tauri/capabilities/default.json  # "compose" in windows list
vite.config.ts                       # compose.html rollup input
src-tauri/src/config.rs              # write_shortcut + collision helper
src-tauri/src/compose.rs             # payload helper, window session, run_clip (new)
src-tauri/src/annotation.rs          # pub cancel_if_open
src-tauri/src/tray.rs                # Write menu item, handle_write, overlay cancel
src-tauri/src/lib.rs                 # mod compose, commands, register/rebind write shortcut
README.md                            # shortcut, tray, settings
docs/screenshots/tray-mockup.html    # Write to daily note item
docs/screenshots/settings-mockup.html
```

---

### Task 1: Config `write_shortcut`

**Files:**
- Modify: `src-tauri/src/config.rs`

**Interfaces:**
- Consumes: existing `AppConfig` serde load/save
- Produces: `AppConfig.write_shortcut: String`; `default_write_shortcut() -> String` returning `"CommandOrControl+Shift+KeyN"`; `AppConfig::write_shortcut_conflicts_with_clip(&self) -> bool`

- [ ] **Step 1: Write the failing tests**

Add these tests inside the existing `mod tests` in `src-tauri/src/config.rs`:

```rust
    #[test]
    fn default_config_includes_write_shortcut() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.write_shortcut, "CommandOrControl+Shift+KeyN");
        assert!(!cfg.write_shortcut_conflicts_with_clip());
    }

    #[test]
    fn load_missing_write_shortcut_uses_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(
            &path,
            r#"{"vault_path":null,"shortcut":"CommandOrControl+Shift+KeyV","text_format":"timestamped","annotation_prompt":true}"#,
        )
        .unwrap();
        let cfg = AppConfig::load(&path).unwrap();
        assert_eq!(cfg.write_shortcut, "CommandOrControl+Shift+KeyN");
    }

    #[test]
    fn write_shortcut_conflicts_when_equal_to_clip() {
        let mut cfg = AppConfig::default();
        cfg.write_shortcut = cfg.shortcut.clone();
        assert!(cfg.write_shortcut_conflicts_with_clip());
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd src-tauri && cargo test --lib config::tests::default_config_includes_write_shortcut config::tests::load_missing_write_shortcut_uses_default config::tests::write_shortcut_conflicts_when_equal_to_clip -- --nocapture`

Expected: FAIL (unknown field `write_shortcut` and/or unknown method)

- [ ] **Step 3: Implement `write_shortcut` on `AppConfig`**

In `src-tauri/src/config.rs`, add the field and helpers:

```rust
    #[serde(default = "default_write_shortcut")]
    pub write_shortcut: String,
```

Place it immediately after `pub shortcut: String`.

Add:

```rust
fn default_write_shortcut() -> String {
    "CommandOrControl+Shift+KeyN".into()
}
```

In `impl Default for AppConfig`, add:

```rust
            write_shortcut: "CommandOrControl+Shift+KeyN".into(),
```

In `impl AppConfig`, add:

```rust
    pub fn write_shortcut_conflicts_with_clip(&self) -> bool {
        self.shortcut == self.write_shortcut
    }
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd src-tauri && cargo test --lib config -- --nocapture`

Expected: PASS (all `config` tests, including existing OCR/annotation default tests)

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/config.rs
git commit -m "feat: add write_shortcut config with default Cmd+Shift+N."
```

---

### Task 2: Compose payload helper

**Files:**
- Create: `src-tauri/src/compose.rs`
- Modify: `src-tauri/src/lib.rs` (add `pub mod compose;`)

**Interfaces:**
- Consumes: none
- Produces: `pub fn compose_payload(text: &str) -> Option<String>` — `None` if `text.trim()` is empty, otherwise `Some(trimmed.to_string())`

- [ ] **Step 1: Write the failing tests**

Create `src-tauri/src/compose.rs` with tests only (the function not yet defined — or define a stub that panics). Prefer writing tests that call `compose_payload`:

```rust
pub fn compose_payload(text: &str) -> Option<String> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::compose_payload;

    #[test]
    fn empty_and_whitespace_are_none() {
        assert_eq!(compose_payload(""), None);
        assert_eq!(compose_payload("   \n\t  "), None);
    }

    #[test]
    fn trims_and_keeps_inner_newlines() {
        assert_eq!(
            compose_payload("  hello\nworld  \n"),
            Some("hello\nworld".to_string())
        );
    }
}
```

Add at the top of `src-tauri/src/lib.rs` (next to `pub mod annotation;`):

```rust
pub mod compose;
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd src-tauri && cargo test --lib compose::tests -- --nocapture`

Expected: FAIL (`unimplemented`)

- [ ] **Step 3: Implement the helper**

Replace the stub:

```rust
pub fn compose_payload(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd src-tauri && cargo test --lib compose -- --nocapture`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/compose.rs src-tauri/src/lib.rs
git commit -m "feat: add compose_payload helper for empty vs insert."
```

---

### Task 3: Compose window shell

**Files:**
- Create: `compose.html`
- Create: `src/compose.css`
- Create: `src/compose.ts`
- Modify: `vite.config.ts`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/capabilities/default.json`

**Interfaces:**
- Consumes: Tauri events `compose-show`; commands `submit_compose` / `cancel_compose` (registered in Task 4)
- Produces: Hidden window label `compose`, URL `compose.html`, size 380×220, `alwaysOnTop: true`, `skipTaskbar: true`, `visible: false`, `resizable: false`

- [ ] **Step 1: Add Vite input**

In `vite.config.ts` `rollupOptions.input`, add:

```ts
        compose: resolve(__dirname, "compose.html"),
```

- [ ] **Step 2: Register the Tauri window**

In `src-tauri/tauri.conf.json`, after the `annotation` window object, add:

```json
      {
        "label": "compose",
        "title": "Write to daily note",
        "url": "compose.html",
        "width": 380,
        "height": 220,
        "visible": false,
        "resizable": false,
        "alwaysOnTop": true,
        "skipTaskbar": true
      },
```

Increase the settings window `"height"` from `560` to `680` so the new shortcut fieldset fits.

In `src-tauri/capabilities/default.json`, set:

```json
  "windows": ["settings", "annotation", "compose", "toast"],
```

- [ ] **Step 3: Create `compose.html`**

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <link rel="stylesheet" href="/src/compose.css" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Write to daily note</title>
    <script type="module" src="/src/compose.ts" defer></script>
  </head>

  <body>
    <main class="compose">
      <textarea
        id="compose-input"
        placeholder="Write to today's note…"
        autocomplete="off"
        spellcheck="true"
      ></textarea>
      <p id="compose-hint" class="hint"></p>
    </main>
  </body>
</html>
```

- [ ] **Step 4: Create `src/compose.css`**

```css
:root {
  font-family: Inter, system-ui, -apple-system, sans-serif;
  font-size: 14px;
  line-height: 1.4;
  color: #1a1a1a;
  background-color: #f5f5f5;
}

* {
  box-sizing: border-box;
}

html,
body {
  margin: 0;
  width: 100%;
  height: 100%;
  overflow: hidden;
}

.compose {
  display: flex;
  flex-direction: column;
  gap: 8px;
  height: 100%;
  padding: 12px;
  overflow: hidden;
}

#compose-input {
  flex: 1 1 auto;
  width: 100%;
  min-height: 0;
  margin: 0;
  font: inherit;
  border-radius: 6px;
  border: 1px solid #c8c8c8;
  padding: 8px 10px;
  background: #fff;
  resize: none;
}

#compose-input:focus {
  outline: none;
  border-color: #2f6feb;
  box-shadow: 0 0 0 2px rgba(47, 111, 235, 0.2);
}

.hint {
  flex: 0 0 16px;
  margin: 0;
  color: #666;
  font-size: 0.8rem;
  line-height: 16px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f0f0f0;
    background-color: #1e1e1e;
  }

  #compose-input {
    color: #f0f0f0;
    background: #2a2a2a;
    border-color: #4a4a4a;
  }

  .hint {
    color: #aaa;
  }
}
```

- [ ] **Step 5: Create `src/compose.ts`**

```ts
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

const inputEl = document.querySelector("#compose-input") as HTMLTextAreaElement;
const hintEl = document.querySelector("#compose-hint") as HTMLParagraphElement;

const isMac = navigator.platform.toUpperCase().includes("MAC");
hintEl.textContent = isMac
  ? "⌘↵ to insert · Esc to cancel"
  : "Ctrl+↵ to insert · Esc to cancel";

window.addEventListener("DOMContentLoaded", () => {
  listen("compose-show", () => {
    inputEl.value = "";
    inputEl.focus();
  });

  inputEl.addEventListener("keydown", (event) => {
    if (event.key === "Escape") {
      event.preventDefault();
      void invoke("cancel_compose");
      return;
    }

    if (event.key !== "Enter" || !(event.metaKey || event.ctrlKey)) {
      return;
    }

    event.preventDefault();
    void invoke("submit_compose", { text: inputEl.value });
  });
});
```

- [ ] **Step 6: Typecheck**

Run: `npx tsc --noEmit`

Expected: PASS (no new errors)

- [ ] **Step 7: Commit**

```bash
git add compose.html src/compose.ts src/compose.css vite.config.ts src-tauri/tauri.conf.json src-tauri/capabilities/default.json
git commit -m "feat: add compose window shell for write-to-note."
```

---

### Task 4: Compose session + annotation `cancel_if_open`

**Files:**
- Modify: `src-tauri/src/compose.rs` (keep `compose_payload` + tests; add window session)
- Modify: `src-tauri/src/annotation.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `compose_payload`, `run_clip`, `ClipboardContent::Text`, `AppConfig`, `tray::flash_tray_success` / `flash_tray_error`
- Produces:
  - `pub struct ComposeState` with `ComposeState::new() -> Self`
  - `pub const COMPOSE_WINDOW_LABEL: &str = "compose"`
  - `pub fn start_compose(app: &AppHandle, config: AppConfig)`
  - `pub fn cancel_if_open(app: &AppHandle)`
  - `#[tauri::command] pub fn submit_compose(app: AppHandle, text: String) -> Result<(), String>`
  - `#[tauri::command] pub fn cancel_compose(app: AppHandle) -> Result<(), String>`
  - `pub fn handle_compose_window_event(window: &Window, event: &WindowEvent)`
  - `annotation::cancel_if_open(app: &AppHandle)` — same abandon behavior as `cancel_annotation`

- [ ] **Step 1: Extract `annotation::cancel_if_open`**

In `src-tauri/src/annotation.rs`, add:

```rust
pub fn cancel_if_open(app: &AppHandle) {
    let state = app.state::<AnnotationState>();
    let id = state.session_id.load(Ordering::SeqCst);
    if state.completed.swap(true, Ordering::SeqCst) {
        if let Some(window) = app.get_webview_window(ANNOTATION_WINDOW_LABEL) {
            let _ = window.hide();
        }
        return;
    }
    abandon_clip(app, id);
}
```

Change `cancel_annotation` to:

```rust
#[tauri::command]
pub fn cancel_annotation(app: AppHandle) -> Result<(), String> {
    cancel_if_open(&app);
    Ok(())
}
```

Replace the `CloseRequested` body in `handle_annotation_window_event` with:

```rust
    if let WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        cancel_if_open(window.app_handle());
    }
```

- [ ] **Step 2: Expand `compose.rs` with the session (keep existing `compose_payload` and tests)**

Append this (imports at the top of the file):

```rust
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Emitter, Manager, Window, WindowEvent};

use crate::clip::service::{run_clip, ClipInput};
use crate::clipboard::ClipboardContent;
use crate::config::AppConfig;
use crate::ocr::health::OcrHealthState;
use crate::platform;
use crate::tray;

pub const COMPOSE_WINDOW_LABEL: &str = "compose";

pub struct ComposeState {
    session_id: AtomicU64,
    completed: AtomicBool,
    pending: Mutex<Option<PendingCompose>>,
}

struct PendingCompose {
    id: u64,
    config: AppConfig,
}

impl ComposeState {
    pub fn new() -> Self {
        Self {
            session_id: AtomicU64::new(0),
            completed: AtomicBool::new(false),
            pending: Mutex::new(None),
        }
    }
}

pub fn start_compose(app: &AppHandle, config: AppConfig) {
    let state = app.state::<ComposeState>();
    let id = state.session_id.fetch_add(1, Ordering::SeqCst) + 1;

    state.completed.store(false, Ordering::SeqCst);
    *state.pending.lock().unwrap() = Some(PendingCompose { id, config });

    let Some(window) = app.get_webview_window(COMPOSE_WINDOW_LABEL) else {
        eprintln!("Compose window not found");
        return;
    };

    let _ = window.emit("compose-show", ());
    let _ = window.center();
    let _ = window.show();
    let _ = window.set_focus();
}

pub fn cancel_if_open(app: &AppHandle) {
    let state = app.state::<ComposeState>();
    let id = state.session_id.load(Ordering::SeqCst);
    if state.completed.swap(true, Ordering::SeqCst) {
        if let Some(window) = app.get_webview_window(COMPOSE_WINDOW_LABEL) {
            let _ = window.hide();
        }
        return;
    }
    abandon(app, id);
}

#[tauri::command]
pub fn submit_compose(app: AppHandle, text: String) -> Result<(), String> {
    let state = app.state::<ComposeState>();
    let id = state.session_id.load(Ordering::SeqCst);
    if state.completed.swap(true, Ordering::SeqCst) {
        return Ok(());
    }

    match compose_payload(&text) {
        None => abandon(&app, id),
        Some(body) => finish_write(&app, id, body),
    }
    Ok(())
}

#[tauri::command]
pub fn cancel_compose(app: AppHandle) -> Result<(), String> {
    cancel_if_open(&app);
    Ok(())
}

fn take_pending(app: &AppHandle, session_id: u64) -> Option<PendingCompose> {
    let state = app.state::<ComposeState>();
    let pending = {
        let mut guard = state.pending.lock().unwrap();
        guard.take()
    };
    pending.filter(|pending| pending.id == session_id)
}

fn hide_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(COMPOSE_WINDOW_LABEL) {
        let _ = window.hide();
    }
}

fn abandon(app: &AppHandle, session_id: u64) {
    let _ = take_pending(app, session_id);
    hide_window(app);
}

fn finish_write(app: &AppHandle, session_id: u64, text: String) {
    let Some(pending) = take_pending(app, session_id) else {
        return;
    };
    hide_window(app);

    let obsidian_json = platform::obsidian_config_path();
    let bundled_eng = app.state::<crate::AppState>().bundled_eng.clone();
    let result = run_clip(ClipInput {
        content: ClipboardContent::Text(text),
        vault_override: pending.config.vault_path.clone(),
        text_format: pending.config.text_format.clone(),
        obsidian_json,
        annotation: None,
        image_ocr: pending.config.image_ocr,
        ocr_languages: pending.config.ocr_languages.clone(),
        tessdata_dir: platform::tessdata_dir(),
        tessdata_prefix: platform::tessdata_prefix(),
        bundled_eng,
        ocr_health: Some(app.state::<Arc<OcrHealthState>>().inner().clone()),
    });

    match result {
        Ok(_) => tray::flash_tray_success(app),
        Err(e) => {
            eprintln!("Write failed: {e}");
            tray::flash_tray_error(app);
        }
    }
}

pub fn handle_compose_window_event(window: &Window, event: &WindowEvent) {
    if window.label() != COMPOSE_WINDOW_LABEL {
        return;
    }

    if let WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        cancel_if_open(window.app_handle());
    }
}
```

- [ ] **Step 3: Wire state, commands, and window events in `lib.rs`**

After `.manage(annotation::AnnotationState::new())` add:

```rust
        .manage(compose::ComposeState::new())
```

In `invoke_handler`, add:

```rust
            compose::submit_compose,
            compose::cancel_compose
```

In `on_window_event`, add:

```rust
            compose::handle_compose_window_event(window, event);
```

In `save_config`, after loading `old_shortcut`, also clone `old_write_shortcut`, and reject collisions **before** saving:

```rust
    if config.write_shortcut_conflicts_with_clip() {
        return Err("Clip and write shortcuts must be different.".into());
    }
    let (old_shortcut, old_write_shortcut) = {
        let current = state.config.lock().unwrap();
        (current.shortcut.clone(), current.write_shortcut.clone())
    };
    config
        .save(&obsclip_config_path())
        .map_err(|e| e.to_string())?;
    *state.config.lock().unwrap() = config.clone();
    rebind_shortcut(&app, &old_shortcut, &config.shortcut)?;
```

Leave write-shortcut rebind for Task 5. Collision reject belongs here so Settings cannot persist an identical pair.

- [ ] **Step 4: Run unit tests**

Run: `cd src-tauri && cargo test --lib`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/compose.rs src-tauri/src/annotation.rs src-tauri/src/lib.rs
git commit -m "feat: compose session submits typed text through run_clip."
```

---

### Task 5: Tray item, `handle_write`, register/rebind write shortcut

**Files:**
- Modify: `src-tauri/src/tray.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `compose::start_compose`, `compose::cancel_if_open`, `annotation::cancel_if_open`, `AppConfig.write_shortcut`
- Produces: `pub fn handle_write(app: &AppHandle)`; tray id `"write"`; `rebind_shortcut` with an `on_press` handler (clip vs write); startup registration that logs on failure instead of aborting setup

- [ ] **Step 1: Add tray item and `handle_write`**

In `src-tauri/src/tray.rs` `setup_tray`:

```rust
    let clip = MenuItem::with_id(handle, "clip", "Clip to daily note", true, None::<&str>)?;
    let write = MenuItem::with_id(handle, "write", "Write to daily note", true, None::<&str>)?;
    let settings = MenuItem::with_id(handle, "settings", "Settings…", true, None::<&str>)?;
    let quit = MenuItem::with_id(handle, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(handle, &[&clip, &write, &settings, &quit])?;
```

In `on_menu_event`:

```rust
            "clip" => handle_clip(app),
            "write" => handle_write(app),
            "settings" => show_settings(app),
            "quit" => app.exit(0),
```

At the start of `handle_clip`, after a successful config load and **only** on the annotation-prompt branch (before `start_clip_with_annotation`), cancel compose:

```rust
    if config.annotation_prompt {
        crate::compose::cancel_if_open(app);
        annotation::start_clip_with_annotation(app, config, content);
        return;
    }
```

Add `handle_write` next to `handle_clip`:

```rust
pub fn handle_write(app: &AppHandle) {
    let config = match AppConfig::load(&platform::obsclip_config_path()) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load config: {e}");
            flash_tray_error(app);
            return;
        }
    };

    annotation::cancel_if_open(app);
    crate::compose::start_compose(app, config);
}
```

- [ ] **Step 2: Register and rebind the write shortcut**

In `src-tauri/src/lib.rs`, extend existing `rebind_shortcut` with an `on_press` handler instead of copying the function. Replace the current 3-argument `rebind_shortcut` with:

```rust
fn rebind_shortcut(
    app: &AppHandle,
    old_shortcut: &str,
    new_shortcut: &str,
    on_press: impl Fn(&AppHandle) + Send + Sync + 'static,
) -> Result<(), String> {
    if old_shortcut == new_shortcut {
        return Ok(());
    }

    let gs = app.global_shortcut();
    if gs.is_registered(old_shortcut) {
        gs.unregister(old_shortcut)
            .map_err(|e| e.to_string())?;
    }

    let app_handle = app.clone();
    gs.on_shortcut(new_shortcut, move |_app, _shortcut, event| {
        if event.state == ShortcutState::Pressed {
            on_press(&app_handle);
        }
    })
    .map_err(|e| e.to_string())?;

    Ok(())
}
```

In `save_config` (Task 4 already captures `old_write_shortcut`), replace the single `rebind_shortcut` call with:

```rust
    rebind_shortcut(&app, &old_shortcut, &config.shortcut, |app| {
        tray::handle_clip(app)
    })?;
    rebind_shortcut(&app, &old_write_shortcut, &config.write_shortcut, |app| {
        tray::handle_write(app)
    })?;
```

In `setup`, after registering the clip shortcut, register write (log, do not `?`):

```rust
            let write_shortcut = config.write_shortcut.clone();
            let write_app = app.handle().clone();
            if let Err(e) = app.handle().global_shortcut().on_shortcut(
                write_shortcut.as_str(),
                move |_app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        tray::handle_write(&write_app);
                    }
                },
            ) {
                eprintln!("Failed to register write shortcut: {e}");
            }
```

- [ ] **Step 3: Compile**

Run: `cd src-tauri && cargo test --lib`

Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/tray.rs src-tauri/src/lib.rs
git commit -m "feat: tray and global shortcut for write to daily note."
```

---

### Task 6: Settings Write shortcut pickers

**Files:**
- Modify: `index.html`
- Modify: `src/main.ts`

**Interfaces:**
- Consumes: `AppConfig.write_shortcut`; `buildShortcut` / `parseShortcut` / `formatShortcutPreview`
- Produces: Settings fieldset **Write shortcut**; auto-save; status error `"Clip and write shortcuts must be different."` when the two built shortcuts are equal; each shortcut still requires at least one modifier

- [ ] **Step 1: Add the Write shortcut fieldset in `index.html`**

Immediately after the existing `</fieldset>` that closes **Global shortcut**, insert:

```html
        <fieldset>
          <legend>Write shortcut</legend>
          <div class="shortcut-fields">
            <label class="field">
              <span>Modifier</span>
              <select id="write-shortcut-primary">
                <option value="CommandOrControl">⌘ Command / Ctrl</option>
                <option value="Alt">⌥ Option / Alt</option>
                <option value="None">None</option>
              </select>
            </label>
            <label class="field">
              <span>Extra</span>
              <select id="write-shortcut-secondary">
                <option value="None">None</option>
                <option value="Shift">⇧ Shift</option>
                <option value="Alt">⌥ Option / Alt</option>
              </select>
            </label>
            <label class="field">
              <span>Key</span>
              <select id="write-shortcut-key"></select>
            </label>
          </div>
          <p class="hint">
            Preview: <kbd id="write-shortcut-preview">⌘⇧N</kbd>
          </p>
        </fieldset>
```

- [ ] **Step 2: Wire TypeScript**

Add `write_shortcut: string` to the `AppConfig` interface.

Add elements:

```ts
let writeShortcutPrimaryEl: HTMLSelectElement;
let writeShortcutSecondaryEl: HTMLSelectElement;
let writeShortcutKeyEl: HTMLSelectElement;
let writeShortcutPreviewEl: HTMLElement;
```

Change `populateKeyOptions` to fill both key selects:

```ts
function populateKeySelect(selectEl: HTMLSelectElement) {
  selectEl.replaceChildren(
    ...LETTERS.map((letter) => {
      const option = document.createElement("option");
      option.value = letter;
      option.textContent = letter;
      return option;
    }),
  );
}

function populateKeyOptions() {
  populateKeySelect(shortcutKeyEl);
  populateKeySelect(writeShortcutKeyEl);
}
```

Add:

```ts
function writeShortcutFromForm(): string {
  const parts: ShortcutParts = {
    primary: writeShortcutPrimaryEl.value as ShortcutParts["primary"],
    secondary: writeShortcutSecondaryEl.value as ShortcutParts["secondary"],
    key: writeShortcutKeyEl.value,
  };
  return buildShortcut(parts);
}

function applyWriteShortcutToForm(shortcut: string) {
  const parts = parseShortcut(shortcut);
  writeShortcutPrimaryEl.value = parts.primary;
  writeShortcutSecondaryEl.value = parts.secondary;
  writeShortcutKeyEl.value = parts.key;
  updateWriteShortcutPreview();
}

function updateWriteShortcutPreview() {
  writeShortcutPreviewEl.textContent = formatShortcutPreview(writeShortcutFromForm());
}

function validateShortcutParts(parts: ShortcutParts, label: string): string | null {
  if (parts.primary !== "None" && parts.secondary !== "None" && parts.primary === parts.secondary) {
    return "Choose different modifiers.";
  }
  if (parts.primary === "None" && parts.secondary === "None") {
    return `Pick at least one modifier for the ${label} shortcut.`;
  }
  return null;
}
```

In `configFromForm`, add `write_shortcut: writeShortcutFromForm()`.

In `applyConfig`, after `applyShortcutToForm(config.shortcut)` add `applyWriteShortcutToForm(config.write_shortcut)`.

Replace the shortcut-validation block in `saveConfig` with:

```ts
  let shortcut: string;
  let writeShortcut: string;
  let clipParts: ShortcutParts;
  let writeParts: ShortcutParts;
  try {
    clipParts = {
      primary: shortcutPrimaryEl.value as ShortcutParts["primary"],
      secondary: shortcutSecondaryEl.value as ShortcutParts["secondary"],
      key: shortcutKeyEl.value,
    };
    writeParts = {
      primary: writeShortcutPrimaryEl.value as ShortcutParts["primary"],
      secondary: writeShortcutSecondaryEl.value as ShortcutParts["secondary"],
      key: writeShortcutKeyEl.value,
    };
    shortcut = buildShortcut(clipParts);
    writeShortcut = buildShortcut(writeParts);
  } catch (error) {
    setStatus(`${error}`, true);
    return;
  }

  const clipError = validateShortcutParts(clipParts, "clip");
  if (clipError) {
    setStatus(clipError, true);
    return;
  }
  const writeError = validateShortcutParts(writeParts, "write");
  if (writeError) {
    setStatus(writeError, true);
    return;
  }

  if (shortcut === writeShortcut) {
    setStatus("Clip and write shortcuts must be different.", true);
    return;
  }
```

Then `const config = configFromForm(); config.shortcut = shortcut; config.write_shortcut = writeShortcut;`

In `DOMContentLoaded`, query the four write-shortcut elements, and attach `change` listeners (preview + `saveConfig`) the same way as the clip shortcut pickers.

- [ ] **Step 3: Typecheck**

Run: `npx tsc --noEmit`

Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add index.html src/main.ts
git commit -m "feat: settings pickers for the write-to-note shortcut."
```

---

### Task 7: README and screenshot mockups

**Files:**
- Modify: `README.md`
- Modify: `docs/screenshots/tray-mockup.html`
- Modify: `docs/screenshots/settings-mockup.html`

**Interfaces:**
- Consumes: shipped UI copy from earlier tasks
- Produces: docs that match the spec (tray order, default write shortcut, compose keys, settings row)

- [ ] **Step 1: Update README usage**

In **Features**, after the one-action clip bullet, add:

```markdown
- **Write to daily note** — global shortcut or tray menu opens a multiline box; typed text is appended with the same text format as clipboard clips
```

In **Usage**, after the clipboard steps, add a paragraph:

```markdown
To type a note instead of clipping the clipboard, press the write shortcut or choose **Write to daily note**. Enter starts a new line; `⌘↵` / `Ctrl+↵` inserts; Esc cancels. Empty submit closes without writing.
```

Add a **Default write shortcut** table next to the existing default shortcut table:

| Platform | Shortcut |
|----------|----------|
| macOS | `⌘⇧N` |
| Windows | `Ctrl+Shift+N` |

In **Tray menu**, insert **Write to daily note** between Clip and Settings.

In **Settings** table, after **Global shortcut**, add:

| **Write shortcut** | Same three pickers for the compose dialog (default `⌘⇧N` / `Ctrl+Shift+N`) |

In **Project structure**, add `src/compose.ts` and `src-tauri/src/compose.rs`.

- [ ] **Step 2: Update mockups**

In `docs/screenshots/tray-mockup.html`, add `<div class="menu-item">Write to daily note</div>` immediately after Clip to daily note. Increase `body` height from `220px` to `260px`.

In `docs/screenshots/settings-mockup.html`, after the Global shortcut fieldset, add a Write shortcut fieldset matching Settings (preview `⌘⇧N`, key `N`, extra Shift).

- [ ] **Step 3: Regenerate screenshots** (if Playwright is available)

```bash
npx playwright screenshot --viewport-size="420,680" \
  file://$PWD/docs/screenshots/settings-mockup.html docs/screenshots/settings.png
npx playwright screenshot --viewport-size="520,260" \
  file://$PWD/docs/screenshots/tray-mockup.html docs/screenshots/tray-menu.png
```

If Playwright is missing, skip screenshot files and leave mockup HTML updated.

- [ ] **Step 4: Commit**

```bash
git add README.md docs/screenshots/tray-mockup.html docs/screenshots/settings-mockup.html docs/screenshots/settings.png docs/screenshots/tray-menu.png
git commit -m "docs: document write-to-daily-note shortcut and tray item."
```

---

## Manual verification (after Task 7)

1. `npm run tauri dev`
2. Write shortcut opens compose; type two lines; `⌘↵` / `Ctrl+↵` appends timestamped (or current format) block to today's note
3. Esc and empty submit do not change the note and do not flash
4. Tray **Write to daily note** does the same
5. Clip shortcut / annotation still work; opening compose while annotation is open dismisses annotation (and the reverse)
6. Settings: change write shortcut, confirm it works; set write equal to clip → error, previous binding kept
