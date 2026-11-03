import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

// Ce fichier s'exécute dans Node : seule `process.env` est utilisée, déclarée ici pour éviter d'ajouter
// les types complets de Node au projet.
declare const process: { env: Record<string, string | undefined> };

// En développement, Vite (port 5173) renvoie les appels /api vers le serveur Rust local
// (`cargo run --bin dev_server`, port 3000). En production, Vercel sert les deux sur le même domaine.
export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      // Autre adresse possible avec la variable SPACEJO_API_URL (par exemple un second serveur local).
      "/api": process.env.SPACEJO_API_URL ?? "http://127.0.0.1:3000",
    },
  },
  test: {
    environment: "node",
  },
});
