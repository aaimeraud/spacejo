// Messages affichés au début et à la fin d'une partie, aux seuls joueurs à thème. Les textes sont dans
// `PLAYER_THEMES` (config.ts), un bloc `messages` par thème. Ce fichier ne fait que choisir le bon message.
import { NAMED_PLAYER_MESSAGES, PLAYER_THEMES, type PlayerMessages } from "./config";

/** Messages du joueur d'après son nom (sans tenir compte des majuscules ni des espaces autour), ou `null`. */
export function playerMessagesFor(name: string): PlayerMessages | null {
  const id = NAMED_PLAYER_MESSAGES[name.trim().toLocaleLowerCase("fr")];
  return id === undefined ? null : PLAYER_THEMES[id].messages;
}

/** Durée d'affichage du bandeau de début, en millisecondes (il se ferme aussi d'un clic). */
export const START_MESSAGE_MS = 6000;

/** Message de fin de partie pour le joueur `me`, d'après les identifiants des gagnants (`view.winners`). */
export function endGameMessage(messages: PlayerMessages, me: number, winners: readonly number[]): string {
  // Un joueur absent de la liste des gagnants a perdu (y compris un joueur éliminé ou qui a abandonné).
  if (!winners.includes(me)) return messages.loser;
  // Plusieurs gagnants : c'est une victoire partagée, pas une victoire seule.
  return winners.length > 1 ? messages.sharedWinner : messages.winner;
}
