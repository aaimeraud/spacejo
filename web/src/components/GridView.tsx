// Grille d'un joueur (colonnes de 3 cartes). Pour sa propre grille, chaque carte est un bouton.
import { useEffect, useRef, useState } from "react";
import type { Action, ColumnView, PlayerView } from "../api/types";
import { clearedColumns } from "../game/columns";
import { slotLabel } from "../game/logic";
import { Card3D, type CardSize } from "./Card";

/** Durée de l'animation d'une colonne retirée (doit correspondre à `column-clear` dans styles.css). */
const COLUMN_CLEAR_MS = 1200;

/** Colonne retirée. Si elle vient de l'être (`clearedValue`), ses trois cartes s'illuminent, s'envolent
 * et disparaissent avant de laisser la place vide. */
function RemovedColumn({
  column,
  clearedValue,
  size,
}: {
  column: number;
  clearedValue: number | undefined;
  size: CardSize;
}) {
  // Valeur figée au montage : l'animation ne se joue qu'une fois, au moment où la colonne disparaît.
  const [ghost, setGhost] = useState(clearedValue);
  useEffect(() => {
    if (ghost === undefined) return;
    const timer = window.setTimeout(() => setGhost(undefined), COLUMN_CLEAR_MS);
    return () => window.clearTimeout(timer);
  }, [ghost]);
  return (
    <div className={`removed-column${ghost !== undefined ? " is-clearing" : ""}`}>
      <span>Colonne {column + 1} retirée</span>
      {ghost !== undefined && (
        <span className="column-ghost" aria-hidden="true">
          {[0, 1, 2].map((r) => (
            <span key={r} className="column-ghost-card" style={{ animationDelay: `${r * 80}ms` }}>
              <Card3D slot={{ kind: "Visible", value: ghost }} size={size} />
            </span>
          ))}
          <span className="column-ghost-label">Colonne identique !</span>
        </span>
      )}
    </div>
  );
}

/** Colonnes retirées depuis l'affichage précédent de cette grille. */
function useClearedColumns(grid: ColumnView[]): Map<number, number> {
  const previous = useRef<ColumnView[] | null>(null);
  const cleared = clearedColumns(previous.current, grid);
  useEffect(() => {
    previous.current = grid;
  });
  return cleared;
}

interface Props {
  player: PlayerView;
  size?: CardSize;
  /** Action d'un clic sur une case, ou `null` si la case n'est pas jouable. Absent : grille d'un autre joueur. */
  actionFor?: (column: number, row: number) => Action | null;
  onAction?: (action: Action) => void;
  disabled?: boolean;
}

export function GridView({ player, size = "md", actionFor, onAction, disabled = false }: Props) {
  const cleared = useClearedColumns(player.grid);
  return (
    <div className={`grid grid-${size}`} role="group" aria-label={`Grille de ${player.name}`}>
      {player.grid.map((column, c) => (
        <div className="grid-column" key={c}>
          {column.kind === "Removed" ? (
            <RemovedColumn column={c} clearedValue={cleared.get(c)} size={size} />
          ) : (
            column.slots.map((slot, r) => {
              const label = slotLabel(slot, c, r);
              if (actionFor === undefined || onAction === undefined) {
                return (
                  <span className="slot" key={r} role="img" aria-label={label}>
                    <Card3D slot={slot} size={size} />
                  </span>
                );
              }
              const action = actionFor(c, r);
              return (
                <button
                  type="button"
                  key={r}
                  className={`slot slot-button${action !== null ? " playable" : ""}`}
                  aria-label={label}
                  disabled={disabled || action === null}
                  onClick={() => {
                    if (action !== null) onAction(action);
                  }}
                >
                  <Card3D slot={slot} size={size} />
                </button>
              );
            })
          )}
        </div>
      ))}
    </div>
  );
}
