// Contenu du tutoriel pour les nouveaux joueurs : une partie aux règles officielles (réglages par défaut), en
// quelques étapes courtes. Le texte suit docs/RULES.md ; les variantes ne sont pas présentées ici (l'hôte les voit
// en créant une partie). Les données sont séparées de l'affichage (components/Tutorial.tsx) pour être testées.
import { CARD_MAX_VALUE, CARD_MIN_VALUE } from "./config";
import { OFFICIAL_RULES } from "./rulesInfo";

/** Valeur d'une carte d'exemple, ou `null` pour une carte face cachée. */
export type TutorialCard = number | null;

/** Illustration d'une étape : une rangée de cartes, ou une grille de joueur (4 colonnes de 3 cartes). */
export type TutorialVisual =
  | { kind: "cards"; cards: TutorialCard[]; caption: string }
  | { kind: "grid"; columns: TutorialCard[][]; highlightColumn?: number; caption: string };

export interface TutorialStep {
  title: string;
  /** Paragraphes courts, lus dans l'ordre. */
  paragraphs: string[];
  visual?: TutorialVisual;
}

/** Plus petite et plus grande valeur d'une carte (docs/RULES.md, « Cartes »). */
export const TUTORIAL_MIN_CARD = CARD_MIN_VALUE;
export const TUTORIAL_MAX_CARD = CARD_MAX_VALUE;

const END_SCORE = OFFICIAL_RULES.end_score;

export const TUTORIAL_STEPS: TutorialStep[] = [
  {
    title: "Le but : avoir le moins de points",
    paragraphs: [
      "Chaque carte vaut de -2 à 12 points. À la fin de chaque manche, on additionne les cartes qui restent devant soi.",
      `La partie s'arrête quand un joueur atteint ${END_SCORE} points : celui qui a le plus petit total gagne.`,
    ],
    visual: { kind: "cards", cards: [-2, 0, 4, 8, 12], caption: "Les petites cartes sont les bonnes." },
  },
  {
    title: "La mise en place",
    paragraphs: [
      "Chacun reçoit 12 cartes face cachée, rangées en 4 colonnes de 3. Personne ne connaît ses cartes, pas même vous.",
      "Au début de la manche, chacun retourne 2 de ses cartes. Celui dont les 2 cartes font le plus de points commence.",
    ],
    visual: {
      kind: "grid",
      columns: [
        [null, 7, null],
        [null, null, null],
        [null, null, 3],
        [null, null, null],
      ],
      caption: "Ici, 7 + 3 = 10 points visibles.",
    },
  },
  {
    title: "À votre tour : piocher",
    paragraphs: [
      "Prenez la carte du dessus de la pioche. Si elle vous plaît, échangez-la contre n'importe quelle carte de votre grille, cachée ou visible.",
      "Sinon, posez-la sur la défausse et retournez une de vos cartes cachées.",
    ],
    visual: { kind: "cards", cards: [null, 5], caption: "La pioche (cachée) et la défausse (visible)." },
  },
  {
    title: "Ou bien : prendre la défausse",
    paragraphs: [
      "Vous pouvez aussi prendre la carte visible de la défausse. Dans ce cas, vous devez l'échanger contre une carte de votre grille.",
      "La carte que vous remplacez part toujours sur la défausse, face visible : tout le monde la voit.",
    ],
  },
  {
    title: "Une colonne identique disparaît",
    paragraphs: [
      "Si les 3 cartes d'une colonne sont visibles et de même valeur, la colonne est retirée : ses points ne comptent plus.",
      "C'est le meilleur moyen de se débarrasser de grosses cartes.",
    ],
    visual: {
      kind: "grid",
      columns: [
        [2, null, 0],
        [9, 9, 9],
        [null, 4, null],
        [null, null, -1],
      ],
      highlightColumn: 1,
      caption: "Ces trois 9 sont retirés du jeu.",
    },
  },
  {
    title: "La fin d'une manche",
    paragraphs: [
      "Dès qu'un joueur a retourné toutes ses cartes, chacun des autres joue un dernier tour. Puis on retourne tout et on compte.",
      "Attention : celui qui a terminé doit avoir le plus petit score de la manche, sinon ses points sont doublés (même en cas d'égalité). Un score nul ou négatif n'est jamais doublé.",
    ],
  },
  {
    title: "La suite",
    paragraphs: [
      "Les scores de chaque manche s'ajoutent. Celui qui a terminé la manche commence la suivante.",
      `Dès qu'un total atteint ${END_SCORE} points, la partie s'arrête à la fin de la manche : le plus petit total gagne. À vous de jouer !`,
    ],
  },
];
