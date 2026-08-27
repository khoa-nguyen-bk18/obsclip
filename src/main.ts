import { invoke } from "@tauri-apps/api/core";
import {
  buildShortcut,
  formatShortcutPreview,
  parseShortcut,
  type ShortcutParts,
} from "./shortcut";

type TextFormat = "timestamped" | "blockquote" | "bullet" | "checkbox";
type LanguageStatus = "bundled" | "installed" | "not_downloaded";

interface AppConfig {
  vault_path: string | null;
  shortcut: string;
  write_shortcut: string;
  text_format: TextFormat;
  annotation_prompt: boolean;
  image_ocr: boolean;
  ocr_languages: string[];
}

interface ResolvedVault {
  path: string | null;
  error: string | null;
}

interface LanguageEntry {
  code: string;
  name: string;
  status: LanguageStatus;
  enabled: boolean;
}

interface OcrHealth {
  message: string | null;
  fix: string | null;
}

let vaultPathEl: HTMLInputElement;
let useDefaultEl: HTMLInputElement;
let savedCustomVaultPath: string | null = null;
let shortcutPrimaryEl: HTMLSelectElement;
let shortcutSecondaryEl: HTMLSelectElement;
let shortcutKeyEl: HTMLSelectElement;
let shortcutPreviewEl: HTMLElement;
let writeShortcutPrimaryEl: HTMLSelectElement;
let writeShortcutSecondaryEl: HTMLSelectElement;
let writeShortcutKeyEl: HTMLSelectElement;
let writeShortcutPreviewEl: HTMLElement;
let textFormatEl: HTMLSelectElement;
let annotationPromptEl: HTMLInputElement;
let imageOcrEl: HTMLInputElement;
let ocrHealthBannerEl: HTMLElement;
let ocrLangSearchEl: HTMLInputElement;
let ocrLangListEl: HTMLElement;
let statusEl: HTMLElement;

let ocrLanguages: string[] = [];
let ocrLanguageEntries: LanguageEntry[] = [];

const LETTERS = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".split("");

const STATUS_LABELS: Record<LanguageStatus, string> = {
  bundled: "Bundled",
  installed: "Installed",
  not_downloaded: "Missing",
};

function setStatus(message: string, isError = false) {
  statusEl.textContent = message;
  statusEl.classList.toggle("error", isError);
}

function syncVaultControls() {
  const useDefault = useDefaultEl.checked;
  document.getElementById("change-vault")!.toggleAttribute("disabled", useDefault);
}

async function refreshVaultDisplay() {
  try {
    const resolved = await invoke<ResolvedVault>("get_resolved_vault_path");
    if (resolved.path) {
      vaultPathEl.value = resolved.path;
      vaultPathEl.placeholder = "";
      vaultPathEl.classList.remove("unresolved");
    } else {
      vaultPathEl.value = "";
      vaultPathEl.placeholder = resolved.error ?? "Vault not configured";
      vaultPathEl.classList.add("unresolved");
    }
  } catch (error) {
    vaultPathEl.value = "";
    vaultPathEl.placeholder = `Failed to resolve vault: ${error}`;
    vaultPathEl.classList.add("unresolved");
  }
}

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

function configFromForm(): AppConfig {
  return {
    vault_path: useDefaultEl.checked ? null : savedCustomVaultPath,
    shortcut: shortcutFromForm(),
    write_shortcut: writeShortcutFromForm(),
    text_format: textFormatEl.value as TextFormat,
    annotation_prompt: annotationPromptEl.checked,
    image_ocr: imageOcrEl.checked,
    ocr_languages: [...ocrLanguages],
  };
}

function applyConfig(config: AppConfig) {
  const useDefault = config.vault_path === null;
  useDefaultEl.checked = useDefault;
  savedCustomVaultPath = config.vault_path;
  applyShortcutToForm(config.shortcut);
  applyWriteShortcutToForm(config.write_shortcut);
  textFormatEl.value = config.text_format;
  annotationPromptEl.checked = config.annotation_prompt;
  imageOcrEl.checked = config.image_ocr;
  ocrLanguages = [...config.ocr_languages];
  syncVaultControls();
  void refreshVaultDisplay();
}

function ocrSearchQuery(): string {
  return ocrLangSearchEl.value.trim().toLowerCase();
}

function matchesOcrSearch(entry: LanguageEntry, query: string): boolean {
  if (!query) {
    return true;
  }
  return (
    entry.name.toLowerCase().includes(query) ||
    entry.code.toLowerCase().includes(query)
  );
}

function renderOcrLanguageList() {
  const query = ocrSearchQuery();
  ocrLangListEl.replaceChildren();

  for (const entry of ocrLanguageEntries) {
    if (!matchesOcrSearch(entry, query)) {
      continue;
    }

    const row = document.createElement("div");
    row.className = "ocr-lang-row";
    row.dataset.code = entry.code;

    const checkboxLabel = document.createElement("label");
    checkboxLabel.className = "ocr-lang-checkbox checkbox";

    const checkbox = document.createElement("input");
    checkbox.type = "checkbox";
    checkbox.checked = entry.enabled;
    checkbox.addEventListener("change", () => {
      void onLanguageToggle(entry.code, checkbox);
    });

    const name = document.createElement("span");
    name.className = "ocr-lang-name";
    name.textContent = `${entry.name} (${entry.code})`;

    checkboxLabel.append(checkbox, name);

    const badge = document.createElement("span");
    badge.className = `ocr-badge ocr-badge-${entry.status.replace("_", "-")}`;
    badge.textContent = STATUS_LABELS[entry.status];

    const actions = document.createElement("div");
    actions.className = "ocr-lang-actions";

    if (entry.status === "not_downloaded" && entry.code !== "eng") {
      const downloadBtn = document.createElement("button");
      downloadBtn.type = "button";
      downloadBtn.textContent = "Download";
      downloadBtn.addEventListener("click", () => {
        void downloadOcrLanguage(entry.code);
      });
      actions.append(downloadBtn);
    }

    if (entry.status === "installed" && entry.code !== "eng") {
      const removeBtn = document.createElement("button");
      removeBtn.type = "button";
      removeBtn.textContent = "Remove";
      removeBtn.addEventListener("click", () => {
        void removeOcrLanguage(entry.code);
      });
      actions.append(removeBtn);
    }

    const meta = document.createElement("div");
    meta.className = "ocr-lang-meta";
    meta.append(badge, actions);

    row.append(checkboxLabel, meta);
    ocrLangListEl.append(row);
  }
}

async function onLanguageToggle(code: string, checkbox: HTMLInputElement) {
  const wasEnabled = ocrLanguages.includes(code);

  if (checkbox.checked) {
    if (!wasEnabled && ocrLanguages.length >= 2) {
      checkbox.checked = false;
      setStatus("Select at most two languages.", true);
      return;
    }
    if (!wasEnabled) {
      ocrLanguages.push(code);
    }
  } else if (wasEnabled) {
    ocrLanguages = ocrLanguages.filter((lang) => lang !== code);
  }

  await saveConfig();
}

async function loadOcrLanguages() {
  try {
    ocrLanguageEntries = await invoke<LanguageEntry[]>("get_ocr_languages");
    renderOcrLanguageList();
  } catch (error) {
    setStatus(`Failed to load OCR languages: ${error}`, true);
  }
}

async function downloadOcrLanguage(code: string) {
  try {
    await invoke("download_ocr_language", { code });
    setStatus(`Downloaded ${code}.`);
    await loadOcrLanguages();
    await loadOcrHealth();
  } catch (error) {
    setStatus(`Failed to download language: ${error}`, true);
  }
}

async function removeOcrLanguage(code: string) {
  try {
    await invoke("remove_ocr_language", { code });
    ocrLanguages = ocrLanguages.filter((lang) => lang !== code);
    setStatus(`Removed ${code}.`);
    await saveConfig();
    await loadOcrLanguages();
    await loadOcrHealth();
  } catch (error) {
    setStatus(`Failed to remove language: ${error}`, true);
  }
}

async function loadOcrHealth() {
  try {
    const health = await invoke<OcrHealth>("get_ocr_health");
    if (health.message) {
      ocrHealthBannerEl.replaceChildren();
      const message = document.createElement("p");
      message.className = "ocr-banner-message";
      message.textContent = health.message;
      ocrHealthBannerEl.append(message);

      if (health.fix) {
        const fix = document.createElement("p");
        fix.className = "ocr-banner-fix";
        fix.textContent = health.fix;
        ocrHealthBannerEl.append(fix);
      }

      ocrHealthBannerEl.classList.remove("hidden");
    } else {
      ocrHealthBannerEl.replaceChildren();
      ocrHealthBannerEl.classList.add("hidden");
    }
  } catch (error) {
    setStatus(`Failed to load OCR health: ${error}`, true);
  }
}

async function loadConfig() {
  try {
    const config = await invoke<AppConfig>("get_config");
    applyConfig(config);
    setStatus("");
    await loadOcrLanguages();
    await loadOcrHealth();
  } catch (error) {
    setStatus(`Failed to load settings: ${error}`, true);
  }
}

async function saveConfig() {
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

  if (ocrLanguages.length > 2) {
    setStatus("Select at most two languages.", true);
    return;
  }

  const config = configFromForm();
  config.shortcut = shortcut;
  config.write_shortcut = writeShortcut;

  try {
    await invoke("save_config", { config });
    applyConfig(config);
    setStatus("Settings saved.");
    await loadOcrLanguages();
  } catch (error) {
    setStatus(`Failed to save: ${error}`, true);
    await loadOcrLanguages();
  }
}

function showVaultFieldError(message: string) {
  vaultPathEl.value = "";
  vaultPathEl.placeholder = message;
  vaultPathEl.classList.add("unresolved");
}

async function changeVault() {
  try {
    const path = await invoke<string | null>("pick_vault_folder");
    if (!path) {
      return;
    }

    try {
      await invoke("validate_obsidian_vault", { path });
    } catch (error) {
      showVaultFieldError(String(error));
      setStatus("");
      return;
    }

    useDefaultEl.checked = false;
    savedCustomVaultPath = path;
    syncVaultControls();
    await saveConfig();
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
  writeShortcutPrimaryEl = document.querySelector("#write-shortcut-primary")!;
  writeShortcutSecondaryEl = document.querySelector("#write-shortcut-secondary")!;
  writeShortcutKeyEl = document.querySelector("#write-shortcut-key")!;
  writeShortcutPreviewEl = document.querySelector("#write-shortcut-preview")!;
  textFormatEl = document.querySelector("#text-format")!;
  annotationPromptEl = document.querySelector("#annotation-prompt")!;
  imageOcrEl = document.querySelector("#image-ocr")!;
  ocrHealthBannerEl = document.querySelector("#ocr-health-banner")!;
  ocrLangSearchEl = document.querySelector("#ocr-lang-search")!;
  ocrLangListEl = document.querySelector("#ocr-lang-list")!;
  statusEl = document.querySelector("#status")!;

  populateKeyOptions();

  useDefaultEl.addEventListener("change", async () => {
    syncVaultControls();
    if (useDefaultEl.checked || savedCustomVaultPath) {
      await saveConfig();
    } else {
      setStatus("");
    }
    await refreshVaultDisplay();
  });
  for (const el of [shortcutPrimaryEl, shortcutSecondaryEl, shortcutKeyEl]) {
    el.addEventListener("change", () => {
      updateShortcutPreview();
      saveConfig();
    });
  }
  for (const el of [writeShortcutPrimaryEl, writeShortcutSecondaryEl, writeShortcutKeyEl]) {
    el.addEventListener("change", () => {
      updateWriteShortcutPreview();
      saveConfig();
    });
  }

  textFormatEl.addEventListener("change", () => saveConfig());
  annotationPromptEl.addEventListener("change", () => saveConfig());
  imageOcrEl.addEventListener("change", () => saveConfig());
  ocrLangSearchEl.addEventListener("input", () => renderOcrLanguageList());
  ocrLangSearchEl.addEventListener("search", () => renderOcrLanguageList());

  document
    .getElementById("settings-form")!
    .addEventListener("submit", (event) => event.preventDefault());

  document
    .querySelector("#change-vault")!
    .addEventListener("click", () => changeVault());

  loadConfig();
});
