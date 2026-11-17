// Thèmes des joueurs (docs/ARCHITECTURE.md) : certains pseudos donnent à leur joueur un thème (`PLAYER_THEMES`).
// Deux niveaux, tous deux purement décoratifs, calculés par l'interface à partir du nom :
// - les cartes et le cadre de la grille du joueur prennent ses couleurs, vus par tous les joueurs ;
// - l'écran entier prend la palette du thème, mais seulement pour ce joueur-là, sur son propre écran.
// Rien ne change au jeu ni à ce que voit le serveur. Les composants ne connaissent aucun thème en particulier :
// ils posent des variables CSS que styles.css utilise.
import type { CSSProperties } from "react";
import { NAMED_PLAYER_THEMES, PLAYER_THEMES, type PlayerTheme } from "../config";

/** Thème d'un joueur d'après son nom (sans tenir compte des majuscules ni des espaces autour), ou `null`. */
export function playerThemeFor(name: string): PlayerTheme | null {
  const id = NAMED_PLAYER_THEMES[name.trim().toLocaleLowerCase("fr")];
  return id === undefined ? null : PLAYER_THEMES[id];
}

/** Les couleurs des cartes en variables CSS (`--theme-…`), à poser sur l'élément qui porte `has-card-theme`. */
function cardStyle(theme: PlayerTheme): CSSProperties {
  return {
    "--theme-back": theme.cards.back,
    "--theme-edge": theme.cards.edge,
    "--theme-mark": theme.cards.mark,
    "--theme-ink": theme.cards.ink,
    "--theme-line-dark": theme.cards.lineOnDark,
    "--theme-line-light": theme.cards.lineOnLight,
  } as CSSProperties;
}

/** Classe et style à poser sur un élément qui prend les couleurs des cartes (rien si pas de thème). */
export function cardThemeProps(theme: PlayerTheme | null): { className: string; style: CSSProperties | undefined } {
  return theme === null ? { className: "", style: undefined } : { className: " has-card-theme", style: cardStyle(theme) };
}

/** Classe et variables de l'écran entier, à poser sur la racine de l'application (rien si pas de thème). */
export function screenThemeProps(theme: PlayerTheme | null): { className: string; style: CSSProperties | undefined } {
  return theme === null
    ? { className: "", style: undefined }
    : { className: " has-screen-theme", style: theme.screen as CSSProperties };
}
