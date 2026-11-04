// Sur téléphone, un seul adversaire est montré en grand. Lequel ? Indépendant du rendu, donc testable.

/**
 * Adversaire à montrer en grand.
 * - `opponents` : identifiants des adversaires, dans l'ordre du tour (après moi) ;
 * - `current` : joueur dont c'est le tour (`null` hors des tours de jeu) ;
 * - `pinned` : adversaire choisi à la main, ou `null` ;
 * - `lastActive` : dernier adversaire qui a joué, ou `null`.
 *
 * Priorité : le choix fait à la main, puis l'adversaire qui joue, puis celui qui vient de jouer (pendant mon
 * tour, on voit encore ce qu'il a fait), puis le premier adversaire encore en jeu.
 */
export function focusedOpponent(
  opponents: { id: number; eliminated: boolean }[],
  current: number | null,
  pinned: number | null,
  lastActive: number | null,
): number | null {
  const isOpponent = (id: number | null): id is number => id !== null && opponents.some((o) => o.id === id);
  if (isOpponent(pinned)) return pinned;
  if (isOpponent(current)) return current;
  if (isOpponent(lastActive)) return lastActive;
  return (opponents.find((o) => !o.eliminated) ?? opponents[0])?.id ?? null;
}
