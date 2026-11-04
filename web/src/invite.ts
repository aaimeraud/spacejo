// Lien d'invitation : l'adresse de l'application suivie du code de la partie (`/?code=K7MPQX`). Ouvert, il
// mène directement à l'étape « Comment vous appelez-vous ? » de la jonction, code déjà rempli. Le lien ne
// contient que le code (jamais de jeton) : c'est la même information que le code donné à voix haute.
import { GAME_CODE_ALPHABET, GAME_CODE_LENGTH, INVITE_CODE_PARAM } from "./config";

/** Lien d'invitation d'une partie, à partir de l'origine du site (`https://exemple.fr`). */
export function inviteLink(origin: string, code: string): string {
  const params = new URLSearchParams({ [INVITE_CODE_PARAM]: code });
  return `${origin}/?${params.toString()}`;
}

/** Code de partie d'une adresse (`location.search`), mis en majuscules ; `null` s'il est absent ou mal
 * formé (le serveur reste seul juge : il refusera de toute façon un code inconnu). */
export function codeFromSearch(search: string): string | null {
  const raw = new URLSearchParams(search).get(INVITE_CODE_PARAM);
  if (raw === null) return null;
  const code = raw.trim().toUpperCase();
  if (code.length !== GAME_CODE_LENGTH) return null;
  return [...code].every((char) => GAME_CODE_ALPHABET.includes(char)) ? code : null;
}
