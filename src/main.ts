import { invoke } from "@tauri-apps/api/core";

type TextFormat = "timestamped" | "blockquote" | "bullet" | "checkbox";

interface AppConfig {
  vault_path: string | null;
  shortcut: string;
  text_format: TextFormat;
}

let vaultPathEl: HTMLInputElement;
let useDefaultEl: HTMLInputElement;
let shortcutEl: HTMLInputElement;
let textFormatEl: HTMLSelectElement;
let statusEl: HTMLElement;

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

function configFromForm(): AppConfig {
  return {
    vault_path: useDefaultEl.checked
      ? null
      : vaultPathEl.value.trim() || null,
    shortcut: shortcutEl.value.trim(),
    text_format: textFormatEl.value as TextFormat,
  };
}

function applyConfig(config: AppConfig) {
  const useDefault = config.vault_path === null;
  useDefaultEl.checked = useDefault;
  vaultPathEl.value = config.vault_path ?? "";
  shortcutEl.value = config.shortcut;
  textFormatEl.value = config.text_format;
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
  const config = configFromForm();
  if (!config.shortcut) {
    setStatus("Shortcut cannot be empty.", true);
    return;
  }

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
  shortcutEl = document.querySelector("#shortcut")!;
  textFormatEl = document.querySelector("#text-format")!;
  statusEl = document.querySelector("#status")!;

  useDefaultEl.addEventListener("change", syncVaultControls);
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
