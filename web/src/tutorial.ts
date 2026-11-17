// Contenu du tutoriel pour les nouveaux joueurs. Il a trois sortes d'étapes :
//  - « info » : du texte (et parfois un dessin), lu ;
//  - « play » : un geste à faire, sur la case, la pioche ou la défausse entourée ;
//  - « bot » : un coup de Aatish, un ami automatisé, joué à la place du joueur.
// La donne et les coups sont fixes, pour que la démonstration soit la même à chaque fois. Les règles suivent
// docs/RULES.md (réglages par défaut). Le moteur qui applique ces gestes est dans `game/tutorialGame.ts`.
import { CARD_MAX_VALUE, CARD_MIN_VALUE } from "./config";
import { applyAction, initialState, type TutorialAction, type TutorialState, type Who } from "./game/tutorialGame";
import { OFFICIAL_RULES } from "./rulesInfo";

/** Valeur d'une carte d'exemple, ou `null` pour une carte face cachée. */
export type TutorialCard = number | null;

/** Illustration d'une étape « info » : une rangée de cartes, ou une grille de joueur (4 colonnes de 3 cartes). */
export type TutorialVisual =
  | { kind: "cards"; cards: TutorialCard[]; caption: string }
  | { kind: "grid"; columns: TutorialCard[][]; caption: string };

/** Cible d'un clic : une case de SA grille, la pioche ou la défausse. */
export type TutorialTarget = { kind: "cell"; column: number; row: number } | { kind: "pile" } | { kind: "discard" };

export interface InfoStep {
  kind: "info";
  title: string;
  /** Paragraphes courts, lus dans l'ordre. */
  paragraphs: string[];
  visual?: TutorialVisual;
}

/** Un geste du joueur. Il faut cliquer sur la cible `targetOf(action)` ; la consigne dit quoi faire. */
export interface PlayStep {
  kind: "play";
  title: string;
  /** Ce qu'il faut faire, lu avant le clic. */
  prompt: string;
  action: TutorialAction;
  /** Ce qui s'est passé, lu après le bon clic. */
  success: string;
}

/** Un coup de Aatish, joué à l'arrivée sur l'étape. `narration` raconte ce qui se passe. */
export interface BotStep {
  kind: "bot";
  title: string;
  actions: TutorialAction[];
  narration: string[];
}

export type TutorialStep = InfoStep | PlayStep | BotStep;

/** Plus petite et plus grande valeur d'une carte (docs/RULES.md, « Cartes »). */
export const TUTORIAL_MIN_CARD = CARD_MIN_VALUE;
export const TUTORIAL_MAX_CARD = CARD_MAX_VALUE;

const END_SCORE = OFFICIAL_RULES.end_score;

/** Cible d'un geste : la case, la pioche ou la défausse. */
export function targetOf(action: TutorialAction): TutorialTarget {
  switch (action.type) {
    case "draw":
      return { kind: "pile" };
    case "takeDiscard":
      return { kind: "discard" };
    case "reveal":
    case "swap":
    case "discardAndReveal":
      return { kind: "cell", column: action.column, row: action.row };
  }
}

export function sameTarget(a: TutorialTarget, b: TutorialTarget): boolean {
  if (a.kind === "cell" && b.kind === "cell") return a.column === b.column && a.row === b.row;
  return a.kind === b.kind;
}

/** Joue un geste sur l'état. Une erreur ici veut dire que le scénario est faux : les tests le détectent. */
export function applyStep(state: TutorialState, step: TutorialStep): TutorialState {
  if (step.kind === "info") return state;
  const who: Who = step.kind === "play" ? "me" : "bot";
  const actions = step.kind === "play" ? [step.action] : step.actions;
  let current = state;
  for (const action of actions) {
    const result = applyAction(current, who, action);
    if (!result.ok) throw new Error(`Scénario du tutoriel invalide (« ${step.title} ») : ${result.error}`);
    current = result.state;
  }
  return current;
}

/** État du plateau juste avant l'étape `index`, en rejouant le scénario depuis la donne. */
export function stateBeforeStep(steps: TutorialStep[], index: number): TutorialState {
  let state = initialState();
  for (const step of steps.slice(0, index)) state = applyStep(state, step);
  return state;
}

export const TUTORIAL_STEPS: TutorialStep[] = [
  {
    kind: "info",
    title: "Le but : avoir le moins de points",
    paragraphs: [
      "Chaque carte vaut de -2 à 12 points. À la fin de chaque manche, on additionne les cartes qui restent devant soi.",
      `La partie s'arrête quand un joueur atteint ${END_SCORE} points : celui qui a le plus petit total gagne.`,
    ],
    visual: { kind: "cards", cards: [-2, 0, 4, 8, 12], caption: "Les petites cartes sont les bonnes." },
  },
  {
    kind: "info",
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
    kind: "info",
    title: "Jouons une manche ensemble",
    paragraphs: [
      "Vous allez jouer une manche complète contre Aatish, un ami automatisé. La donne est toujours la même, pour que vous puissiez tester chaque geste.",
      "La case à cliquer est entourée en pointillés. Si vous cliquez ailleurs, un message vous le signale : rien n'est perdu.",
    ],
  },
  // Mise en place : vous retournez deux cartes de la colonne 2, puis Aatish retourne les siennes.
  {
    kind: "play",
    title: "Mise en place : retournez une carte",
    prompt: "Cliquez sur la première carte de la colonne 2 (en haut) pour la retourner.",
    action: { type: "reveal", column: 1, row: 0 },
    success: "Vous avez retourné un 9.",
  },
  {
    kind: "play",
    title: "Mise en place : retournez une deuxième carte",
    prompt: "Retournez la carte juste en dessous, toujours dans la colonne 2.",
    action: { type: "reveal", column: 1, row: 1 },
    success: "Deux 9 dans la colonne 2. Si la troisième carte de cette colonne est aussi un 9, la colonne sera retirée.",
  },
  {
    kind: "bot",
    title: "Aatish retourne deux cartes",
    actions: [
      { type: "reveal", column: 0, row: 0 },
      { type: "reveal", column: 1, row: 0 },
    ],
    narration: [
      "Aatish retourne un 2 et un 1 : 3 points en tout.",
      "Vous avez 18 points, Aatish en a 3 : c'est vous qui commencez, car le plus grand total commence.",
    ],
  },
  {
    kind: "info",
    title: "À votre tour",
    paragraphs: [
      "Vous avez deux possibilités : prendre la carte cachée du dessus de la pioche, ou prendre la carte visible du dessus de la défausse.",
      "La carte prise doit ensuite être échangée avec une carte de votre grille. Une carte piochée peut aussi être défaussée : vous retournez alors une carte cachée.",
    ],
  },
  // Premier tour : piocher un 6, puis l'échanger contre la carte du coin en haut à gauche.
  {
    kind: "play",
    title: "Piocher",
    prompt: "Cliquez sur la pioche pour prendre la carte du dessus.",
    action: { type: "draw" },
    success: "Vous avez un 6 en main. Vous le gardez : il va remplacer une carte de votre grille.",
  },
  {
    kind: "play",
    title: "Échanger",
    prompt: "Cliquez sur la carte du coin en haut à gauche : elle est échangée avec le 6.",
    action: { type: "swap", column: 0, row: 0 },
    success: "Le 10 part sur la défausse, face visible. Votre 6 prend sa place, visible lui aussi.",
  },
  {
    kind: "bot",
    title: "Le tour de Aatish",
    actions: [
      { type: "draw" },
      { type: "discardAndReveal", column: 0, row: 1 },
    ],
    narration: [
      "Aatish pioche un 3 et le défausse.",
      "Au lieu de le garder, Aatish retourne une de ses cartes cachées : un 5.",
    ],
  },
  // Défausser : vous piochez un 7, qui ne vous plaît pas, puis vous le défaussez en retournant une carte cachée.
  {
    kind: "play",
    title: "Piocher, puis défausser",
    prompt: "Cliquez sur la pioche.",
    action: { type: "draw" },
    success: "Un 7 : il ne vous plaît pas. Vous allez le défausser en retournant une carte cachée.",
  },
  {
    kind: "play",
    title: "Défausser et retourner",
    prompt: "Cliquez sur la deuxième carte de la colonne 1 (cachée) : elle est retournée, et le 7 part sur la défausse.",
    action: { type: "discardAndReveal", column: 0, row: 1 },
    success: "Vous avez retourné un 4. Le 7 est maintenant le dessus de la défausse.",
  },
  {
    kind: "bot",
    title: "Le tour de Aatish",
    actions: [
      { type: "draw" },
      { type: "swap", column: 1, row: 1 },
    ],
    narration: [
      "Aatish pioche un 1 et le garde, pour le placer à la place d'une carte cachée de sa colonne 2.",
      "La carte remplacée, un 4, part sur la défausse, face visible.",
    ],
  },
  // Prendre la défausse : vous prenez le 4 visible et l'échangez contre votre 11 caché.
  {
    kind: "play",
    title: "Prendre la défausse",
    prompt: "Cliquez sur la défausse pour prendre la carte visible du dessus (un 4).",
    action: { type: "takeDiscard" },
    success: "Vous avez pris le 4. Vous devez l'échanger tout de suite avec une carte de votre grille.",
  },
  {
    kind: "play",
    title: "Échanger la carte prise à la défausse",
    prompt: "Cliquez sur la carte du milieu de la colonne 4 : elle est échangée avec le 4.",
    action: { type: "swap", column: 3, row: 1 },
    success: "La carte cachée remplacée, un 11, part sur la défausse, face visible. Votre 4 est visible dans la grille.",
  },
  {
    kind: "bot",
    title: "Le tour de Aatish",
    actions: [
      { type: "draw" },
      { type: "discardAndReveal", column: 2, row: 0 },
    ],
    narration: [
      "Aatish pioche un 2 et le défausse.",
      "Aatish retourne une carte cachée : un 0.",
    ],
  },
  // Colonne identique : vous piochez un 9, qui fait trois 9 dans la colonne 2.
  {
    kind: "play",
    title: "Piocher le troisième 9",
    prompt: "Cliquez sur la pioche.",
    action: { type: "draw" },
    success: "Un 9 ! Il y a déjà deux 9 visibles dans la colonne 2.",
  },
  {
    kind: "play",
    title: "Une colonne identique disparaît",
    prompt: "Cliquez sur la carte du bas de la colonne 2 (cachée) : elle est échangée avec le 9.",
    action: { type: "swap", column: 1, row: 2 },
    success:
      "Trois 9 de même valeur : la colonne 2 est retirée. Ses 3 cartes vont sur la défausse, et elle ne compte plus dans votre total.",
  },
  {
    kind: "info",
    title: "La fin d'une manche",
    paragraphs: [
      "Dès qu'un joueur a retourné toutes ses cartes, chacun des autres joue un dernier tour. Puis on retourne tout et on compte.",
      "Attention : celui qui a terminé doit avoir le plus petit score de la manche, sinon ses points sont doublés (même en cas d'égalité). Un score nul ou négatif n'est jamais doublé.",
    ],
  },
  {
    kind: "info",
    title: "La suite",
    paragraphs: [
      "Ici, la démonstration s'arrête : dans une vraie partie, la manche continue jusqu'à ce que quelqu'un retourne toutes ses cartes.",
      `Les scores de chaque manche s'ajoutent. Dès qu'un total atteint ${END_SCORE} points, la partie s'arrête à la fin de la manche : le plus petit total gagne. À vous de jouer !`,
    ],
  },
];
