// Bouton qui bascule entre le thème clair et le thème sombre ; le choix est gardé sur cet appareil.
import { Moon, Sun } from "lucide-react";
import { useState } from "react";
import { applyTheme, currentTheme, saveTheme } from "../theme";

export function ThemeToggle() {
  const [theme, setTheme] = useState(currentTheme);
  const next = theme === "dark" ? "light" : "dark";
  const label = next === "light" ? "Passer au thème clair" : "Passer au thème sombre";
  return (
    <button
      type="button"
      className="ghost-button icon-only"
      aria-label={label}
      title={label}
      onClick={() => {
        applyTheme(next);
        saveTheme(window.localStorage, next);
        setTheme(next);
      }}
    >
      {next === "light" ? <Sun aria-hidden="true" /> : <Moon aria-hidden="true" />}
    </button>
  );
}
