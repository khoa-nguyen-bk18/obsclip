import { listen } from "@tauri-apps/api/event";

interface ToastShowPayload {
  message: string;
}

const messageEl = document.querySelector("#toast-message") as HTMLDivElement;

window.addEventListener("DOMContentLoaded", () => {
  listen<ToastShowPayload>("toast-show", (event) => {
    messageEl.textContent = event.payload.message;
  });
});
