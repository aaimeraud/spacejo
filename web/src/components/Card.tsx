// Visuel d'une carte (dessin original). La valeur est toujours écrite en chiffres ; la couleur
// n'est qu'un indice supplémentaire (accessibilité : jamais la couleur seule).
import type { PublicSlot } from "../api/types";

/** Famille de couleur selon la valeur : négatif, zéro, faible, moyen, élevé. */
export function valueTone(value: number): string {
  if (value < 0) return "tone-negative";
  if (value === 0) return "tone-zero";
  if (value <= 4) return "tone-low";
  if (value <= 8) return "tone-mid";
  return "tone-high";
}

export function CardFace({ slot }: { slot: PublicSlot }) {
  if (slot.kind === "Hidden") {
    return (
      <span className="card card-hidden" aria-hidden="true">
        <span className="card-star">✦</span>
      </span>
    );
  }
  return (
    <span className={`card ${valueTone(slot.value)}`} aria-hidden="true">
      <span className="card-value">{slot.value}</span>
    </span>
  );
}

/** Carte isolée (pioche, défausse, carte en main). */
export function ValueCard({ value }: { value: number | null }) {
  return <CardFace slot={value === null ? { kind: "Hidden" } : { kind: "Visible", value }} />;
}
