// Carte en relief (CSS 3D, dessin original). Deux faces : le dos étoilé et la face visible. Retourner
// une carte, c'est faire pivoter l'ensemble de 180° autour de l'axe vertical (classe `is-up`).
// La valeur est toujours écrite en chiffres ; la couleur n'est qu'un repère supplémentaire.
import type { PublicSlot } from "../api/types";

/** Famille de couleur selon la valeur : négatif, zéro, faible, moyen, élevé. */
export function valueTone(value: number): string {
  if (value < 0) return "tone-negative";
  if (value === 0) return "tone-zero";
  if (value <= 4) return "tone-low";
  if (value <= 8) return "tone-mid";
  return "tone-high";
}

export type CardSize = "sm" | "md" | "lg";

interface Props {
  slot: PublicSlot;
  size?: CardSize;
}

export function Card3D({ slot, size = "md" }: Props) {
  const faceUp = slot.kind === "Visible";
  return (
    <span className={`card3d card3d-${size}${faceUp ? " is-up" : ""}`} aria-hidden="true">
      <span className="card3d-inner">
        <span className="card3d-face card3d-back">
          <span className="card-orbit" />
          <span className="card-star">✦</span>
        </span>
        {faceUp ? (
          // `key` sur la valeur : quand une carte visible est remplacée par une autre, la face est
          // recréée et rejoue son animation d'apparition.
          <span key={slot.value} className={`card3d-face card3d-front ${valueTone(slot.value)}`}>
            <span className="card-corner card-corner-top">{slot.value}</span>
            <span className="card-value">{slot.value}</span>
            <span className="card-corner card-corner-bottom">{slot.value}</span>
          </span>
        ) : (
          <span className="card3d-face card3d-front card3d-blank" />
        )}
      </span>
    </span>
  );
}

/** Carte isolée (pioche, défausse, carte en main) : `null` = face cachée. */
export function ValueCard({ value, size = "lg" }: { value: number | null; size?: CardSize }) {
  return <Card3D slot={value === null ? { kind: "Hidden" } : { kind: "Visible", value }} size={size} />;
}
