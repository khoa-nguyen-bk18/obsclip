export type TauriModifier = "CommandOrControl" | "Shift" | "Alt" | "Control";

export interface ShortcutParts {
  primary: TauriModifier | "None";
  secondary: TauriModifier | "None";
  key: string;
}

const MODIFIER_ORDER: TauriModifier[] = [
  "CommandOrControl",
  "Control",
  "Alt",
  "Shift",
];

const KEY_PATTERN = /^Key([A-Z])$/;

export function parseShortcut(shortcut: string): ShortcutParts {
  const fallback: ShortcutParts = {
    primary: "CommandOrControl",
    secondary: "Shift",
    key: "V",
  };

  const tokens = shortcut.split("+").map((part) => part.trim()).filter(Boolean);
  if (tokens.length === 0) {
    return fallback;
  }

  const keyToken = tokens[tokens.length - 1];
  const keyMatch = keyToken.match(KEY_PATTERN);
  if (!keyMatch) {
    return fallback;
  }

  const modifiers = tokens.slice(0, -1).filter(isModifier);
  const primary =
    modifiers.find((modifier) => modifier === "CommandOrControl" || modifier === "Control") ??
    modifiers.find((modifier) => modifier === "Alt") ??
    "None";
  const secondary =
    modifiers.find(
      (modifier) =>
        modifier !== primary &&
        (modifier === "Shift" || modifier === "Alt" || modifier === "Control"),
    ) ?? "None";

  return {
    primary,
    secondary,
    key: keyMatch[1],
  };
}

export function buildShortcut(parts: ShortcutParts): string {
  const tokens: string[] = [];

  if (parts.primary !== "None") {
    tokens.push(parts.primary);
  }
  if (parts.secondary !== "None" && parts.secondary !== parts.primary) {
    tokens.push(parts.secondary);
  }

  const key = parts.key.toUpperCase();
  if (!/^[A-Z]$/.test(key)) {
    throw new Error("Key must be a single letter A–Z");
  }

  tokens.push(`Key${key}`);
  return tokens.join("+");
}

export function formatShortcutPreview(shortcut: string): string {
  const isMac = navigator.platform.toLowerCase().includes("mac");
  const parts = parseShortcut(shortcut);
  const labels: string[] = [];

  if (parts.primary === "CommandOrControl") {
    labels.push(isMac ? "⌘" : "Ctrl");
  } else if (parts.primary === "Control") {
    labels.push(isMac ? "⌃" : "Ctrl");
  } else if (parts.primary === "Alt") {
    labels.push(isMac ? "⌥" : "Alt");
  }

  if (parts.secondary === "Shift") {
    labels.push(isMac ? "⇧" : "Shift");
  } else if (parts.secondary === "Alt") {
    labels.push(isMac ? "⌥" : "Alt");
  } else if (parts.secondary === "Control") {
    labels.push(isMac ? "⌃" : "Ctrl");
  }

  labels.push(parts.key.toUpperCase());
  return labels.join(isMac ? "" : "+");
}

function isModifier(token: string): token is TauriModifier {
  return MODIFIER_ORDER.includes(token as TauriModifier);
}
