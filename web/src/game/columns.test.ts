import { describe, expect, it } from "vitest";
import type { ColumnView } from "../api/types";
import { clearedColumns } from "./columns";

const col = (...values: (number | null)[]): ColumnView => ({
  kind: "Present",
  slots: values.map((v) => (v === null ? { kind: "Hidden" } : { kind: "Visible", value: v })),
});
const removed: ColumnView = { kind: "Removed" };

describe("clearedColumns", () => {
  it("repère une colonne qui vient d'être retirée et sa valeur", () => {
    const before = [col(7, 7, 3), col(1, null, 2)];
    const after = [removed, col(1, null, 2)];
    expect([...clearedColumns(before, after)]).toEqual([[0, 7]]);
  });

  it("fonctionne quand la troisième carte était cachée", () => {
    expect([...clearedColumns([col(null, -1, -1)], [removed])]).toEqual([[0, -1]]);
  });

  it("ignore les colonnes déjà retirées et le premier affichage", () => {
    expect(clearedColumns([removed], [removed]).size).toBe(0);
    expect(clearedColumns(null, [removed]).size).toBe(0);
  });
});
