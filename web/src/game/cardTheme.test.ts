import { describe, expect, it } from "vitest";
import { CARD_THEMES, NAMED_CARD_THEMES } from "../config";
import { cardThemeFor, cardThemeProps } from "./cardTheme";

describe("thèmes de cartes personnalisés", () => {
  it("donne son thème à chaque pseudo de la liste, sans tenir compte des majuscules ni des espaces autour", () => {
    for (const [name, id] of Object.entries(NAMED_CARD_THEMES)) {
      expect(cardThemeFor(name)).toBe(CARD_THEMES[id]);
      expect(cardThemeFor(`  ${name.toUpperCase()} `)).toBe(CARD_THEMES[id]);
    }
  });

  it("laisse tous les autres noms au thème normal", () => {
    for (const name of ["Alice", "MissLegend", "MissLegendR2", "Miss LegendR", ""]) {
      expect(cardThemeFor(name)).toBeNull();
      expect(cardThemeProps(cardThemeFor(name))).toEqual({ className: "", style: undefined });
    }
  });

  it("chaque thème déclare des couleurs valides (#rrggbb), utilisables telles quelles en CSS", () => {
    for (const theme of Object.values(CARD_THEMES)) {
      for (const color of Object.values(theme)) expect(color).toMatch(/^#[0-9a-f]{6}$/i);
    }
  });

  it("pose les couleurs du thème dans des variables CSS", () => {
    const { className, style } = cardThemeProps(CARD_THEMES.pink);
    expect(className).toBe(" has-card-theme");
    expect(style).toMatchObject({ "--theme-back": CARD_THEMES.pink.back, "--theme-line-light": CARD_THEMES.pink.lineOnLight });
  });
});
