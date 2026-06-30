import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface AnnotationShowPayload {
  entryPreview: string;
}

const inputEl = document.querySelector("#annotation-input") as HTMLInputElement;
const hintEl = document.querySelector("#annotation-hint") as HTMLParagraphElement;
const previewEl = document.querySelector("#entry-preview") as HTMLDivElement;

function previewOneLine(text: string): string {
  const line = text.split(/\r?\n/).find((part) => part.trim().length > 0);
  return line ?? text;
}

const isMac = navigator.platform.toUpperCase().includes("MAC");
hintEl.textContent = isMac
  ? "⌘↵ to clip · Esc to cancel"
  : "Ctrl+↵ to clip · Esc to cancel";

window.addEventListener("DOMContentLoaded", () => {
  listen<AnnotationShowPayload>("annotation-show", (event) => {
    inputEl.value = "";
    previewEl.textContent = previewOneLine(event.payload.entryPreview);
    inputEl.focus();
  });

  inputEl.addEventListener("keydown", (event) => {
    if (event.key === "Escape") {
      event.preventDefault();
      void invoke("cancel_annotation");
      return;
    }

    if (event.key !== "Enter" || !(event.metaKey || event.ctrlKey)) {
      return;
    }

    event.preventDefault();
    void invoke("submit_annotation", { text: inputEl.value });
  });
});
