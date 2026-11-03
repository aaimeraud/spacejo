// Grille d'un joueur (colonnes de 3 cartes). Pour sa propre grille, chaque carte est un bouton.
import type { Action, PlayerView } from "../api/types";
import { slotLabel } from "../game/logic";
import { Card3D, type CardSize } from "./Card";

interface Props {
  player: PlayerView;
  size?: CardSize;
  /** Action d'un clic sur une case, ou `null` si la case n'est pas jouable. Absent : grille d'un autre joueur. */
  actionFor?: (column: number, row: number) => Action | null;
  onAction?: (action: Action) => void;
  disabled?: boolean;
}

export function GridView({ player, size = "md", actionFor, onAction, disabled = false }: Props) {
  return (
    <div className={`grid grid-${size}`} role="group" aria-label={`Grille de ${player.name}`}>
      {player.grid.map((column, c) => (
        <div className="grid-column" key={c}>
          {column.kind === "Removed" ? (
            <div className="removed-column">
              <span>Colonne {c + 1} retirée</span>
            </div>
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
