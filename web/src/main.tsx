import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import "./styles.css";
import { initTheme } from "./theme";

initTheme();

// Service worker (application installable) : seulement sur le site construit, pour ne pas garder en cache
// les fichiers du serveur de développement.
if (import.meta.env.PROD && "serviceWorker" in navigator) {
  window.addEventListener("load", () => {
    navigator.serviceWorker.register("/sw.js").catch(() => {
      // Sans service worker, le site marche tout autant ; il n'est simplement pas installable partout.
    });
  });
}

const root = document.getElementById("root");
if (root !== null) {
  createRoot(root).render(
    <StrictMode>
      <App />
    </StrictMode>,
  );
}
