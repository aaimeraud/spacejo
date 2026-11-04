import { describe, expect, it } from "vitest";
import { qrMatrix } from "./qr";

describe("code QR", () => {
  const matrix = qrMatrix("https://spacejo.example/?code=K7MPQX");

  it("est carré, d'une taille de code QR valide (21 + 4 × n cases)", () => {
    const size = matrix.length;
    expect(matrix.every((row) => row.length === size)).toBe(true);
    expect((size - 21) % 4).toBe(0);
  });

  it("a les trois motifs de repérage dans les coins", () => {
    const size = matrix.length;
    // Coin haut gauche d'un motif : bord sombre de 7 cases, puis un anneau clair, puis un carré sombre au centre.
    for (const [top, left] of [
      [0, 0],
      [0, size - 7],
      [size - 7, 0],
    ] as const) {
      expect(matrix[top]?.[left]).toBe(true);
      expect(matrix[top + 1]?.[left + 1]).toBe(false);
      expect(matrix[top + 3]?.[left + 3]).toBe(true);
    }
  });

  it("dépend du texte encodé", () => {
    expect(qrMatrix("https://spacejo.example/?code=ABCDEF")).not.toEqual(matrix);
  });
});
