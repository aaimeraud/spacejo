// Détection des colonnes qui viennent d'être retirées (pour leur animation). Ne décide d'aucune règle :
// on compare seulement la vue précédente et la nouvelle, reçues du serveur.
import type { ColumnView } from "../api/types";

/** Colonnes présentes avant et retirées maintenant, avec la valeur de leurs 3 cartes identiques.
 * La valeur est celle qui revient le plus souvent parmi les cartes visibles de l'ancienne colonne : au moins
 * deux d'entre elles avaient déjà la valeur finale (un seul coup change une seule carte). */
export function clearedColumns(before: ColumnView[] | null, after: ColumnView[]): Map<number, number> {
  const cleared = new Map<number, number>();
  if (before === null) return cleared;
  after.forEach((column, c) => {
    const previous = before[c];
    if (column.kind !== "Removed" || previous === undefined || previous.kind !== "Present") return;
    const counts = new Map<number, number>();
    for (const slot of previous.slots) {
      if (slot.kind === "Visible") counts.set(slot.value, (counts.get(slot.value) ?? 0) + 1);
    }
    let best: number | undefined;
    let bestCount = 0;
    for (const [value, count] of counts) {
      if (count > bestCount) {
        best = value;
        bestCount = count;
      }
    }
    if (best !== undefined) cleared.set(c, best);
  });
  return cleared;
}
