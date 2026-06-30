import { invoke } from "@tauri-apps/api/core";
import {
  buildShortcut,
  formatShortcutPreview,
  parseShortcut,
  type ShortcutParts,
} from "./shortcut";

type TextFormat = "timestamped" | "blockquote" | "bullet" | "checkbox";

interface AppConfig {
  vault_path: string | null;
  shortcut: string;
  text_format: TextFormat;
  annotation_prompt: boolean;
}

let vaultPathEl: HTMLInputElement;
let useDefaultEl: HTMLInputElement;
let shortcutPrimaryEl: HTMLSelectElement;
let shortcutSecondaryEl: HTMLSelectElement;
let shortcutKeyEl: HTMLSelectElement;
let shortcutPreviewEl: HTMLElement;
let textFormatEl: HTMLSelectElement;
let annotationPromptEl: HTMLInputElement;
let statusEl: HTMLElement;

const LETTERS = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".split("");

function setStatus(message: string, isError = false) {
  statusEl.textContent = message;
  statusEl.classList.toggle("error", isError);
}

function syncVaultControls() {
  const useDefault = useDefaultEl.checked;
  vaultPathEl.disabled = useDefault;
  document.getElementById("browse-vault")!.toggleAttribute("disabled", useDefault);
  if (useDefault) {
    vaultPathEl.value = "";
  }
}

function populateKeyOptions() {
  shortcutKeyEl.replaceChildren(
    ...LETTERS.map((letter) => {
      const option = document.createElement("option");
      option.value = letter;
      option.textContent = letter;
      return option;
    }),
  );
}

function shortcutFromForm(): string {
  const parts: ShortcutParts = {
    primary: shortcutPrimaryEl.value as ShortcutParts["primary"],
    secondary: shortcutSecondaryEl.value as ShortcutParts["secondary"],
    key: shortcutKeyEl.value,
  };
  return buildShortcut(parts);
}

function applyShortcutToForm(shortcut: string) {
  const parts = parseShortcut(shortcut);
  shortcutPrimaryEl.value = parts.primary;
  shortcutSecondaryEl.value = parts.secondary;
  shortcutKeyEl.value = parts.key;
  updateShortcutPreview();
}

function updateShortcutPreview() {
  shortcutPreviewEl.textContent = formatShortcutPreview(shortcutFromForm());
}

function configFromForm(): AppConfig {
  return {
    vault_path: useDefaultEl.checked
      ? null
      : vaultPathEl.value.trim() || null,
    shortcut: shortcutFromForm(),
    text_format: textFormatEl.value as TextFormat,
    annotation_prompt: annotationPromptEl.checked,
  };
}

function applyConfig(config: AppConfig) {
  const useDefault = config.vault_path === null;
  useDefaultEl.checked = useDefault;
  vaultPathEl.value = config.vault_path ?? "";
  applyShortcutToForm(config.shortcut);
  textFormatEl.value = config.text_format;
  annotationPromptEl.checked = config.annotation_prompt;
  syncVaultControls();
}

async function loadConfig() {
  try {
    const config = await invoke<AppConfig>("get_config");
    applyConfig(config);
    setStatus("");
  } catch (error) {
    setStatus(`Failed to load settings: ${error}`, true);
  }
}

async function saveConfig() {
  let shortcut: string;
  let parts: ShortcutParts;
  try {
    parts = {
      primary: shortcutPrimaryEl.value as ShortcutParts["primary"],
      secondary: shortcutSecondaryEl.value as ShortcutParts["secondary"],
      key: shortcutKeyEl.value,
    };
    shortcut = buildShortcut(parts);
  } catch (error) {
    setStatus(`${error}`, true);
    return;
  }

  if (parts.primary !== "None" && parts.secondary !== "None" && parts.primary === parts.secondary) {
    setStatus("Choose different modifiers.", true);
    return;
  }

  if (parts.primary === "None" && parts.secondary === "None") {
    setStatus("Pick at least one modifier for the shortcut.", true);
    return;
  }

  const config = configFromForm();
  config.shortcut = shortcut;

  try {
    await invoke("save_config", { config });
    applyConfig(config);
    setStatus("Settings saved.");
  } catch (error) {
    setStatus(`Failed to save: ${error}`, true);
  }
}

async function browseVault() {
  try {
    const path = await invoke<string | null>("pick_vault_folder");
    if (path) {
      useDefaultEl.checked = false;
      vaultPathEl.value = path;
      syncVaultControls();
    }
  } catch (error) {
    setStatus(`Failed to pick folder: ${error}`, true);
  }
}

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

  useDefaultEl.addEventListener("change", syncVaultControls);
  for (const el of [shortcutPrimaryEl, shortcutSecondaryEl, shortcutKeyEl]) {
    el.addEventListener("change", updateShortcutPreview);
  }

  document
    .querySelector("#browse-vault")!
    .addEventListener("click", () => browseVault());
  document
    .querySelector("#settings-form")!
    .addEventListener("submit", (event) => {
      event.preventDefault();
      saveConfig();
    });

  loadConfig();
});
