// Repérer le moment où la défausse est remélangée pour refaire la pioche (réglage `empty_pile` =
// `ReshuffleDiscard`), pour l'animer. Le serveur ne l'annonce pas : pendant une même manche, la pioche ne fait que
// diminuer, sauf justement quand la défausse redevient la pioche.
import type { GameView } from "../api/types";

/** La pioche vient-elle d'être refaite avec la défausse, entre ces deux vues ? */
export function pileReshuffled(previous: GameView | null, next: GameView): boolean {
  if (previous === null) return false;
  // Nouvelle manche : la pioche est redistribuée, ce n'est pas un remélange.
  if (previous.round !== next.round || previous.phase === "Lobby") return false;
  return next.draw_pile_count > previous.draw_pile_count;
}
