import { describe, expect, it } from "vitest";
import { NAMED_PLAYER_THEMES, PLAYER_THEMES } from "../config";
import { cardThemeProps, playerThemeFor, screenThemeProps } from "./cardTheme";

describe("thèmes des joueurs", () => {
  it("donne son thème à chaque pseudo de la liste, sans tenir compte des majuscules ni des espaces autour", () => {
    for (const [name, id] of Object.entries(NAMED_PLAYER_THEMES)) {
      expect(playerThemeFor(name)).toBe(PLAYER_THEMES[id]);
      expect(playerThemeFor(`  ${name.toUpperCase()} `)).toBe(PLAYER_THEMES[id]);
    }
  });

  it("ne donne le thème (cartes et écran) qu'aux deux pseudos principaux, pas à ceux qui n'ont que des messages", () => {
    for (const name of ["mass", "massky", "dan", "daniel", "aaimeraud"]) {
      expect(playerThemeFor(name)).toBeNull();
    }
  });

  it("laisse tous les autres noms sans thème", () => {
    for (const name of ["Alice", "MissLegend", "MissLegendR2", "Miss LegendR", ""]) {
      expect(playerThemeFor(name)).toBeNull();
      expect(cardThemeProps(playerThemeFor(name))).toEqual({ className: "", style: undefined });
      expect(screenThemeProps(playerThemeFor(name))).toEqual({ className: "", style: undefined });
    }
  });

  it("chaque thème déclare des couleurs de cartes valides (#rrggbb)", () => {
    for (const theme of Object.values(PLAYER_THEMES)) {
      for (const color of Object.values(theme.cards)) expect(color).toMatch(/^#[0-9a-f]{6}$/i);
    }
  });

  it("chaque thème déclare une palette d'écran complète, avec les mêmes variables que :root", () => {
    const keys = Object.keys(PLAYER_THEMES.kitty.screen).sort();
    expect(keys).toContain("--bg");
    expect(keys).toContain("--text");
    for (const theme of Object.values(PLAYER_THEMES)) {
      expect(Object.keys(theme.screen).sort()).toEqual(keys);
    }
  });

  it("pose les couleurs des cartes dans des variables CSS", () => {
    const { className, style } = cardThemeProps(PLAYER_THEMES.kitty);
    expect(className).toBe(" has-card-theme");
    expect(style).toMatchObject({ "--theme-back": PLAYER_THEMES.kitty.cards.back, "--theme-line-light": PLAYER_THEMES.kitty.cards.lineOnLight });
  });

  it("pose la palette de l'écran entier avec sa classe", () => {
    const { className, style } = screenThemeProps(PLAYER_THEMES.daniel);
    expect(className).toBe(" has-screen-theme");
    expect(style).toMatchObject({ "--bg": PLAYER_THEMES.daniel.screen["--bg"] });
  });
});
