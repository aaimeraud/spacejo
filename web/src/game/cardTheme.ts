// Thèmes de cartes personnalisés (docs/ARCHITECTURE.md) : certains pseudos donnent au dos des cartes du joueur, à ses
// contours de cartes et à ses bulles du chat les couleurs d'un thème de `CARD_THEMES`. Purement décoratif : calculé
// par l'interface à partir du nom, sans rien changer au jeu ni à ce que voit le serveur. Les composants ne connaissent
// aucun thème en particulier : ils posent les couleurs du thème dans des variables CSS, que styles.css utilise.
import type { CSSProperties } from "react";
import { CARD_THEMES, NAMED_CARD_THEMES, type CardTheme } from "../config";

/** Thème de cartes d'un joueur d'après son nom (sans tenir compte des majuscules ni des espaces autour), ou `null`. */
export function cardThemeFor(name: string): CardTheme | null {
  const id = NAMED_CARD_THEMES[name.trim().toLocaleLowerCase("fr")];
  return id === undefined ? null : CARD_THEMES[id];
}

/** Les couleurs du thème en variables CSS (`--theme-…`), à poser sur l'élément qui porte la classe `has-card-theme`. */
export function cardThemeStyle(theme: CardTheme): CSSProperties {
  return {
    "--theme-back": theme.back,
    "--theme-edge": theme.edge,
    "--theme-mark": theme.mark,
    "--theme-ink": theme.ink,
    "--theme-line-dark": theme.lineOnDark,
    "--theme-line-light": theme.lineOnLight,
  } as CSSProperties;
}

/** Classe et style à poser sur un élément thématisé (rien si le joueur n'a pas de thème). */
export function cardThemeProps(theme: CardTheme | null): { className: string; style: CSSProperties | undefined } {
  return theme === null ? { className: "", style: undefined } : { className: " has-card-theme", style: cardThemeStyle(theme) };
}
