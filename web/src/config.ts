// Constantes de l'interface, définies une seule fois.

/** Intervalle du polling de l'état pendant une partie, onglet visible (en millisecondes). */
export const POLL_INTERVAL_MS = 2000;

/** Intervalle au salon : on n'y attend que l'arrivée des joueurs et le lancement. */
export const POLL_LOBBY_INTERVAL_MS = 3000;

/** Intervalle quand l'onglet est caché (autre onglet, téléphone en veille) ou la partie finie. Il reste sous
 * le délai d'absence du serveur (`PRESENCE_TIMEOUT_SECS`, 15 s) : un joueur qui change d'onglet n'est pas
 * compté absent. Chaque requête est un appel de fonction facturé : ralentir ici réduit nettement le coût. */
export const POLL_IDLE_INTERVAL_MS = 10000;

/** Bornes de `end_score` (mêmes valeurs que `END_SCORE_MIN` / `END_SCORE_MAX` du core ; le serveur
 * reste seul juge et refuse toute valeur hors bornes). */
export const END_SCORE_MIN = 10;
export const END_SCORE_MAX = 500;

/** Longueur maximale d'un nom de joueur (même valeur que `MAX_NAME_CHARS` du core). */
export const MAX_NAME_CHARS = 20;

/** Longueur d'un code de partie (même valeur que `GAME_CODE_LENGTH` de l'API). */
export const GAME_CODE_LENGTH = 6;

/** Clé de `sessionStorage` où l'on garde la session (code, jeton, identifiant). `sessionStorage`
 * est propre à chaque onglet : on peut jouer à deux dans deux onglets du même navigateur. */
export const SESSION_STORAGE_KEY = "spacejo.session";

/** Clé de `localStorage` des parties rejointes sur cet appareil, pour pouvoir les reprendre après avoir
 * fermé l'onglet. */
export const KNOWN_SESSIONS_KEY = "spacejo.sessions";

/** Nombre maximal de parties mémorisées pour la reprise. */
export const MAX_KNOWN_SESSIONS = 5;


/** Nombre maximal de joueurs (même valeur que `MAX_PLAYERS` du core ; le serveur reste seul juge). */
export const MAX_PLAYERS = 10;

/** Clé de `localStorage` du thème choisi (« light » ou « dark ») ; absente, on suit le réglage du système. */
export const THEME_KEY = "spacejo.theme";
