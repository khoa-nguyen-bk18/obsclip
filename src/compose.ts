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
