import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

// En développement, Vite (port 5173) renvoie les appels /api vers le serveur Rust local
// (`cargo run --bin dev_server`, port 3000). En production, Vercel sert les deux sur le même domaine.
export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      "/api": "http://127.0.0.1:3000",
    },
  },
  test: {
    environment: "node",
  },
});
