# Settings Auto-Save & Vault Folder Picker Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan step-by-step. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the Save button from Settings; auto-save every change immediately; replace editable vault path with read-only display + Change… folder picker.

**Architecture:** Event-driven `change` listeners on all form controls call the existing `saveConfig()` function. Vault path is set only via native folder picker. No backend changes.

**Tech Stack:** Tauri 2, TypeScript (Vite frontend), vanilla HTML/CSS

**Spec:** `docs/superpowers/specs/2026-07-01-settings-auto-save-design.md`

---

### Task 1: Update vault UI in HTML

**Files:**
- Modify: `index.html`

- [ ] **Step 1: Replace editable vault path input with read-only display**

In the Vault fieldset, replace the text input + Browse button row with:

```html
<div class="row">
  <input
    id="vault-path"
    type="text"
    readonly
    placeholder="Using Obsidian default"
    autocomplete="off"
  />
  <button type="button" id="change-vault">Change…</button>
</div>
```

- [ ] **Step 2: Remove Save button, keep status line**

Replace the actions div with:

```html
<div class="actions">
  <p id="status" class="status" aria-live="polite"></p>
</div>
```

Remove `type="submit"` semantics — the form no longer submits.

---

### Task 2: Update styles for read-only path and remove Save button styles

**Files:**
- Modify: `src/styles.css`

- [ ] **Step 1: Add read-only vault path styles**

After the `input:disabled` block, add:

```css
input[readonly] {
  color: inherit;
  background: #f8f8f8;
  cursor: default;
  text-overflow: ellipsis;
  overflow: hidden;
  white-space: nowrap;
}
```

- [ ] **Step 2: Remove `#save-btn` styles**

Delete the `#save-btn`, `#save-btn:hover`, and dark-mode `#save-btn` rules.

- [ ] **Step 3: Add dark-mode read-only input style**

Inside `@media (prefers-color-scheme: dark)`, add:

```css
input[readonly] {
  background: #2a2a2a;
}
```

---

### Task 3: Wire auto-save and vault picker in main.ts

**Files:**
- Modify: `src/main.ts`

- [ ] **Step 1: Update `syncVaultControls` for read-only display**

```typescript
function syncVaultControls() {
  const useDefault = useDefaultEl.checked;
  vaultPathEl.toggleAttribute("disabled", useDefault);
  document.getElementById("change-vault")!.toggleAttribute("disabled", useDefault);
  if (useDefault) {
    vaultPathEl.value = "";
    vaultPathEl.placeholder = "Using Obsidian default";
  } else {
    vaultPathEl.placeholder = "No folder selected";
  }
}
```

- [ ] **Step 2: Rename `browseVault` to `changeVault` and auto-save after pick**

```typescript
async function changeVault() {
  try {
    const path = await invoke<string | null>("pick_vault_folder");
    if (path) {
      useDefaultEl.checked = false;
      vaultPathEl.value = path;
      syncVaultControls();
      await saveConfig();
    }
  } catch (error) {
    setStatus(`Failed to pick folder: ${error}`, true);
  }
}
```

- [ ] **Step 3: Replace DOMContentLoaded event wiring**

Remove form submit handler. Add auto-save listeners:

```typescript
window.addEventListener("DOMContentLoaded", () => {
  vaultPathEl = document.querySelector("#vault-path")!;
  useDefaultEl = document.querySelector("#use-obsidian-default")!;
  shortcutPrimaryEl = document.querySelector("#shortcut-primary")!;
  shortcutSecondaryEl = document.querySelector("#shortcut-secondary")!;
  shortcutKeyEl = document.querySelector("#shortcut-key")!;
  shortcutPreviewEl = document.querySelector("#shortcut-preview")!;
  textFormatEl = document.querySelector("#text-format")!;
  annotationPromptEl = document.querySelector("#annotation-prompt")!;
  statusEl = document.querySelector("#status")!;

  populateKeyOptions();

  useDefaultEl.addEventListener("change", () => {
    syncVaultControls();
    saveConfig();
  });

  for (const el of [shortcutPrimaryEl, shortcutSecondaryEl, shortcutKeyEl]) {
    el.addEventListener("change", () => {
      updateShortcutPreview();
      saveConfig();
    });
  }

  textFormatEl.addEventListener("change", () => saveConfig());
  annotationPromptEl.addEventListener("change", () => saveConfig());

  document
    .querySelector("#change-vault")!
    .addEventListener("click", () => changeVault());

  loadConfig();
});
```

Note: `changeVault` already calls `saveConfig()` after pick, so no duplicate save needed from a separate listener.

- [ ] **Step 4: Remove `disabled` from `syncVaultControls` vault path toggle**

Use `readonly` attribute in HTML; control enabled state via `disabled` only when "Use Obsidian default" is checked (field is non-interactive). The `toggleAttribute("disabled", useDefault)` on vaultPathEl remains so the field appears greyed when using default.

---

### Task 4: Update settings mockup

**Files:**
- Modify: `docs/screenshots/settings-mockup.html`

- [ ] **Step 1: Sync mockup with new vault UI and remove Save button**

Match `index.html` structure: readonly vault path, Change… button, no Save button.

---

### Task 5: Manual verification

- [ ] **Step 1: Run dev build**

```bash
npm run tauri dev
```

- [ ] **Step 2: Verify auto-save scenarios**

1. Open Settings — fields load, no Save button visible
2. Toggle "Use Obsidian default" — status shows "Saved"
3. Click Change… — pick folder — path displays, saves
4. Change text format — saves on change
5. Set invalid shortcut — error shown, clip shortcut unchanged
6. Set valid shortcut — saves, new shortcut works

- [ ] **Step 3: Commit**

```bash
git add index.html src/main.ts src/styles.css docs/
git commit -m "feat: auto-save settings and replace vault path with folder picker"
```
