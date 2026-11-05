// Constantes de l'interface, définies une seule fois.

/** Intervalle du polling de l'état pendant une partie, onglet visible (en millisecondes). */
export const POLL_INTERVAL_MS = 2000;

/** Intervalle au salon : on n'y attend que l'arrivée des joueurs et le lancement. */
export const POLL_LOBBY_INTERVAL_MS = 3000;

/** Intervalle quand l'onglet est caché (autre onglet, téléphone en veille) ou la partie finie. Il reste sous
 * le délai d'absence du serveur (`PRESENCE_TIMEOUT_SECS`, 15 s) : un joueur qui change d'onglet n'est pas
 * compté absent. Ralentir ici ménage le serveur et la base. */
export const POLL_IDLE_INTERVAL_MS = 10000;

/** Intervalle quand le flux d'avis du serveur est ouvert : les coups arrivent par ce flux, le polling ne sert
 * plus qu'à signaler sa présence (sous les 15 s du délai d'absence) et à rattraper un avis perdu. */
export const POLL_LIVE_INTERVAL_MS = 5000;

/** Bornes de `end_score` (mêmes valeurs que `END_SCORE_MIN` / `END_SCORE_MAX` du core ; le serveur
 * reste seul juge et refuse toute valeur hors bornes). */
export const END_SCORE_MIN = 10;
export const END_SCORE_MAX = 500;

/** Longueur maximale d'un nom de joueur (même valeur que `MAX_NAME_CHARS` du core). */
export const MAX_NAME_CHARS = 20;

/** Longueur d'un code de partie (même valeur que `GAME_CODE_LENGTH` de l'API). */
export const GAME_CODE_LENGTH = 6;

/** Alphabet des codes de partie, sans caractères ambigus (ni 0/O ni 1/I) : même valeur que l'API. */
export const GAME_CODE_ALPHABET = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

/** Paramètre d'adresse du lien d'invitation (`/?code=K7MPQX`). */
export const INVITE_CODE_PARAM = "code";

/** Nouveaux essais d'un retournement de mise en place refusé pour conflit de version (409). */
export const MAX_REVEAL_RETRIES = 3;

/** Durée d'affichage d'un message d'information (« X a abandonné la partie »), en millisecondes. */
export const NOTICE_DURATION_MS = 6000;

/** Durée de l'animation du remélange de la défausse en pioche (en millisecondes). */
export const RESHUFFLE_ANIMATION_MS = 2200;

/** Longueur maximale d'un message du chat, en caractères (même valeur que `MAX_CHAT_CHARS` de l'API ; le serveur
 * reste seul juge). */
export const MAX_CHAT_CHARS = 200;

/** Réactions aux messages du chat, dans l'ordre du sélecteur : nom envoyé à l'API (`CHAT_REACTIONS` de
 * `src/handlers.rs`), emoji affiché, et libellé lu par les lecteurs d'écran. La première est celle du double tap. */
export const CHAT_REACTIONS = [
  { name: "heart", emoji: "❤️", label: "cœur" },
  { name: "thumbs_up", emoji: "👍", label: "pouce levé" },
  { name: "laugh", emoji: "😂", label: "rire" },
  { name: "wow", emoji: "😮", label: "surprise" },
  { name: "sad", emoji: "😢", label: "tristesse" },
  { name: "pray", emoji: "🙏", label: "merci" },
] as const;

/** Réaction posée par un double tap / double clic sur un message. */
export const QUICK_REACTION = "heart";

/** Deux appuis sur un message à moins de ce délai (millisecondes) forment un double tap. */
export const DOUBLE_TAP_MS = 350;

/** Nombre de messages du chat gardés à l'écran (les plus récents). */
export const MAX_CHAT_KEPT = 100;

/** Rafraîchissement de la liste des parties publiques sur l'accueil (en millisecondes). */
export const PUBLIC_GAMES_REFRESH_MS = 5000;

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

/** Dépôt public du jeu sur GitHub (version et dernier commit affichés en bas de l'accueil). */
export const GITHUB_REPO = "aaimeraud/spacejo";

/** Branche dont on montre le dernier commit. */
export const GITHUB_BRANCH = "main";

/** Adresse de l'API REST de GitHub. */
export const GITHUB_API = "https://api.github.com";

/** Durée pendant laquelle la version et le dernier commit lus sur GitHub sont gardés (10 minutes) : l'API publique
 * n'accepte que 60 requêtes par heure et par adresse. */
export const GITHUB_CACHE_MS = 10 * 60 * 1000;

/** Clé de `localStorage` de ce cache. */
export const GITHUB_CACHE_KEY = "spacejo.github";
