// Thème clair ou sombre. Par défaut, celui du système (`prefers-color-scheme`) ; un choix explicite de
// l'utilisateur est gardé dans `localStorage` et l'emporte. Le thème s'applique par l'attribut
// `data-theme` de <html>, que lit styles.css.
import { THEME_KEY } from "./config";
import type { KeyValueStorage } from "./session";

export type Theme = "light" | "dark";

/** Thème choisi explicitement, ou `null` s'il faut suivre le système. */
export function storedTheme(storage: KeyValueStorage): Theme | null {
  try {
    const value = storage.getItem(THEME_KEY);
    return value === "light" || value === "dark" ? value : null;
  } catch {
    return null;
  }
}

/** Thème à afficher : le choix gardé, sinon celui du système. */
export function resolveTheme(stored: Theme | null, systemPrefersLight: boolean): Theme {
  return stored ?? (systemPrefersLight ? "light" : "dark");
}

export function saveTheme(storage: KeyValueStorage, theme: Theme): void {
  try {
    storage.setItem(THEME_KEY, theme);
  } catch {
    // Stockage indisponible : le thème vaut pour cette visite seulement.
  }
}

/** Applique le thème à la page (lu par styles.css). */
export function applyTheme(theme: Theme): void {
  document.documentElement.dataset.theme = theme;
}

/** Thème affiché en ce moment (celui appliqué à <html>). */
export function currentTheme(): Theme {
  return document.documentElement.dataset.theme === "light" ? "light" : "dark";
}

/** Au démarrage : applique le thème, puis suit les changements du système tant que rien n'est choisi. */
export function initTheme(): void {
  const media = window.matchMedia("(prefers-color-scheme: light)");
  applyTheme(resolveTheme(storedTheme(window.localStorage), media.matches));
  media.addEventListener("change", (event) => {
    if (storedTheme(window.localStorage) === null) applyTheme(event.matches ? "light" : "dark");
  });
}
