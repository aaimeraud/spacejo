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

/** Durées de conservation annoncées sur la page de confidentialité : mêmes valeurs que `CHAT_KEPT_AFTER_GAME_OVER` et
 * `GAME_KEPT_WHEN_IDLE` de l'API (src/handlers.rs), qui seule les applique. */
export const CHAT_KEPT_AFTER_GAME_OVER_MINUTES = 15;
export const GAME_KEPT_WHEN_IDLE_HOURS = 24;

/** Un thème de cartes personnalisé : les couleurs du dos des cartes d'un joueur (avec un cœur à la place du « S »), de
 * ses contours de cartes et de ses bulles du chat (docs/ARCHITECTURE.md, « Thèmes de cartes personnalisés »). */
export interface CardTheme {
  /** Fond du dos des cartes et des bulles du chat (une couleur foncée). */
  back: string;
  /** Bord du dos et cadre intérieur, un peu plus clair que le fond. */
  edge: string;
  /** Cœur du dos et nom de l'auteur dans le chat (une couleur claire). */
  mark: string;
  /** Texte des bulles du chat. */
  ink: string;
  /** Contours (cartes jouables, colonne retirée) sur le fond sombre du site : plutôt clair. */
  lineOnDark: string;
  /** Les mêmes contours sur le fond clair du site : plutôt foncé. */
  lineOnLight: string;
}

/** Les thèmes de cartes. Pour en ajouter un : une entrée ici, puis un ou plusieurs pseudos dans `NAMED_CARD_THEMES`.
 * `satisfies` vérifie chaque entrée sans perdre la liste exacte des noms de thème (`CardThemeId`). */
export const CARD_THEMES = {
  pink: {
    back: "#6e1f42",
    edge: "#9c3a66",
    mark: "#ffb8d4",
    ink: "#ffe8f1",
    lineOnDark: "#ff8fbd",
    lineOnLight: "#c2306e",
  },
  blue: {
    back: "#1b3566",
    edge: "#3b5f9e",
    mark: "#b9d3ff",
    ink: "#e6f0ff",
    lineOnDark: "#7fb0ff",
    lineOnLight: "#2b5fc7",
  },
} satisfies Record<string, CardTheme>;

export type CardThemeId = keyof typeof CARD_THEMES;

/** Pseudos (en minuscules) qui reçoivent un thème de cartes. Rien sur le site ne l'annonce. */
export const NAMED_CARD_THEMES: Readonly<Record<string, CardThemeId>> = {
  misslegendr: "pink",
  misterlegendr: "blue",
};

/** Plus petite et plus grande valeur d'une carte (docs/RULES.md). */
export const CARD_MIN_VALUE = -2;
export const CARD_MAX_VALUE = 12;

/** Nombre de cartes de l'éventail décoratif de l'accueil. */
export const HOME_CARD_COUNT = 5;

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

/** Adresse de contact de l'éditeur (mentions légales, demandes sur les données personnelles). */
export const CONTACT_EMAIL = "danielfortune.pro@proton.me";

/** Branche dont on montre le dernier commit. */
export const GITHUB_BRANCH = "main";

/** Adresse de l'API REST de GitHub. */
export const GITHUB_API = "https://api.github.com";

/** Durée pendant laquelle la version et le dernier commit lus sur GitHub sont gardés (10 minutes) : l'API publique
 * n'accepte que 60 requêtes par heure et par adresse. */
export const GITHUB_CACHE_MS = 10 * 60 * 1000;

/** Clé de `localStorage` de ce cache. */
export const GITHUB_CACHE_KEY = "spacejo.github";
