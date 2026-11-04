// Textes d'explication des réglages (docs/RULES.md, « Règles maison »).
// L'interface affiche les réglages REÇUS du serveur ; ce fichier ne fournit que les phrases
// en français et les étiquettes « officiel » / « variante » (voir ASSUMPTIONS.md, n° 13).
import { END_SCORE_MAX, END_SCORE_MIN } from "./config";
import type { Rules } from "./api/types";

/** Règles officielles : mêmes valeurs que `Rules::default()` du core. Sert au formulaire de création
 * et au bouton « Règles officielles ». */
export const OFFICIAL_RULES: Rules = {
  penalty: "DoubleUnlessStrictlyLowest",
  penalty_scope: "RoundTotal",
  first_player_tie: "JoinOrder",
  next_round_starter: "Finisher",
  empty_pile: "ReshuffleDiscard",
  end_tie: "SharedVictory",
  end_score: 100,
  deck: "Full",
};

export type Status = "officiel" | "variante";

export interface OptionInfo<V extends string> {
  value: V;
  label: string;
  explanation: string;
  /** Exemple concret, avec des noms et des nombres (texte fixe : l'interface ne calcule aucun score). */
  example: string;
  status: Status;
}

/** Réglages dont la valeur est une énumération. */
export type ChoiceKey = Exclude<keyof Rules, "end_score">;

export interface ChoiceInfo<K extends ChoiceKey> {
  key: K;
  title: string;
  /** Titre court, pour la carte du réglage. */
  short: string;
  /** Petit pictogramme textuel (décoratif). */
  icon: string;
  /** Mention affichée quand la notice officielle ne tranche pas. */
  note?: string;
  options: OptionInfo<Rules[K]>[];
}

const UNSPECIFIED = "La notice officielle ne précise pas ce cas.";

export const PENALTY: ChoiceInfo<"penalty"> = {
  key: "penalty",
  short: "Pénalité",
  icon: "×2",
  title: "Pénalité du joueur qui termine la manche",
  options: [
    {
      value: "DoubleUnlessStrictlyLowest",
      label: "Doublé s'il n'est pas seul le plus bas",
      explanation:
        "Son score de manche est doublé s'il n'a pas le score strictement le plus bas (une égalité le double aussi).",
      example: "Vous terminez avec 12 et un adversaire a aussi 12 : vous marquez 24.",
      status: "officiel",
    },
    {
      value: "DoubleOnlyIfBeaten",
      label: "Doublé seulement s'il est battu",
      explanation: "Son score est doublé seulement si un autre joueur a strictement moins ; une égalité l'épargne.",
      example: "Vous terminez avec 12 : à égalité avec un adversaire, vous marquez 12 ; s'il a 10, vous marquez 24.",
      status: "variante",
    },
    {
      value: "Disabled",
      label: "Aucune pénalité",
      explanation: "Terminer la manche n'est jamais pénalisé.",
      example: "Vous terminez avec 12 et un adversaire a 5 : vous marquez quand même 12.",
      status: "variante",
    },
  ],
};

export const PENALTY_SCOPE: ChoiceInfo<"penalty_scope"> = {
  key: "penalty_scope",
  short: "Portée de la pénalité",
  icon: "±",
  title: "Ce que double la pénalité",
  note: "Les sources divergent ; le livret indique le total de la manche.",
  options: [
    {
      value: "RoundTotal",
      label: "Le total de la manche",
      explanation: "Le score de manche entier est doublé. Un total nul ou négatif n'est jamais doublé.",
      example: "Cartes 10, -2 et 4 (total 12), avec pénalité : 24.",
      status: "officiel",
    },
    {
      value: "PositiveCardsOnly",
      label: "Les cartes positives seulement",
      explanation:
        "Les cartes positives comptent deux fois, les négatives une seule fois. Un total nul ou négatif n'est jamais doublé.",
      example: "Cartes 10, -2 et 4 (total 12), avec pénalité : 26 (le 10 et le 4 comptent double).",
      status: "variante",
    },
  ],
};

export const FIRST_PLAYER_TIE: ChoiceInfo<"first_player_tie"> = {
  key: "first_player_tie",
  short: "Égalité au départ",
  icon: "1er",
  title: "Égalité pour commencer la première manche",
  note: UNSPECIFIED,
  options: [
    {
      value: "JoinOrder",
      label: "Ordre d'arrivée",
      explanation: "Parmi les ex æquo, le premier arrivé dans la partie commence (l'hôte d'abord).",
      example: "Alice et Bob ont tous deux 9 : Alice, arrivée avant, commence.",
      status: "variante",
    },
    {
      value: "Random",
      label: "Tirage au sort",
      explanation: "Le serveur tire au sort parmi les ex æquo.",
      example: "Alice et Bob ont tous deux 9 : le serveur tire l'un des deux au sort.",
      status: "variante",
    },
  ],
};

export const NEXT_ROUND_STARTER: ChoiceInfo<"next_round_starter"> = {
  key: "next_round_starter",
  short: "Manche suivante",
  icon: "↻",
  title: "Qui commence la manche suivante",
  options: [
    {
      value: "Finisher",
      label: "Celui qui a terminé",
      explanation: "Le joueur qui a terminé la manche précédente commence la suivante.",
      example: "Bob a retourné toutes ses cartes le premier : il commence la manche suivante.",
      status: "officiel",
    },
    {
      value: "RoundWinner",
      label: "Le gagnant de la manche",
      explanation:
        "Le joueur au score de manche le plus bas (après pénalité ; en cas d'égalité, le premier arrivé) commence.",
      example: "Chloé a le plus petit score de la manche : elle commence la suivante.",
      status: "variante",
    },
    {
      value: "Rotate",
      label: "Chacun son tour",
      explanation: "Le joueur suivant celui qui a commencé la manche précédente commence.",
      example: "Alice a commencé cette manche : Bob, assis après elle, commence la suivante.",
      status: "variante",
    },
  ],
};

export const EMPTY_PILE: ChoiceInfo<"empty_pile"> = {
  key: "empty_pile",
  short: "Pioche vide",
  icon: "∅",
  title: "Pioche épuisée",
  note: UNSPECIFIED,
  options: [
    {
      value: "ReshuffleDiscard",
      label: "Remélanger la défausse",
      explanation:
        "La défausse est remélangée pour former une nouvelle pioche ; sa carte du dessus reste visible.",
      example: "Plus de pioche : la défausse, sauf sa carte du dessus, est mélangée et devient la pioche.",
      status: "variante",
    },
    {
      value: "EndRound",
      label: "Fin de manche immédiate",
      explanation: "La manche s'arrête tout de suite et on compte, sans dernier tour et sans pénalité.",
      example: "Plus de pioche : tout le monde retourne ses cartes et on compte, sans dernier tour.",
      status: "variante",
    },
  ],
};

export const DECK: ChoiceInfo<"deck"> = {
  key: "deck",
  short: "Taille de la pioche",
  icon: "▤",
  title: "Nombre de cartes en jeu",
  options: [
    {
      value: "Full",
      label: "Paquet complet",
      explanation: "Les 150 cartes sont utilisées à chaque manche, quel que soit le nombre de joueurs.",
      example: "À 2 joueurs, 24 cartes sont distribuées et la pioche en compte 125 : elle ne s'épuise presque jamais.",
      status: "officiel",
    },
    {
      value: "Reduced",
      label: "Pioche réduite",
      explanation:
        "On garde 40 cartes par joueur, tirées au hasard (12 pour sa grille, 28 pour la pioche ; tout le paquet à partir de 4 joueurs) : la pioche s'épuise plus vite et la défausse est remélangée plus souvent.",
      example: "À 2 joueurs, 40 cartes en jeu : 24 distribuées, 1 sur la défausse, 15 dans la pioche.",
      status: "variante",
    },
  ],
};

export const END_TIE: ChoiceInfo<"end_tie"> = {
  key: "end_tie",
  short: "Égalité finale",
  icon: "=",
  title: "Égalité en fin de partie",
  note: UNSPECIFIED,
  options: [
    {
      value: "SharedVictory",
      label: "Victoire partagée",
      explanation: "Tous les joueurs à égalité sur le total le plus bas gagnent.",
      example: "Alice et Bob finissent tous deux à 87 : ils gagnent ensemble.",
      status: "variante",
    },
    {
      value: "ExtraRound",
      label: "Manche supplémentaire",
      explanation: "On rejoue des manches tant que le total le plus bas n'est pas unique.",
      example: "Alice et Bob finissent tous deux à 87 : on joue une manche de plus.",
      status: "variante",
    },
  ],
};

/** Tous les réglages à choix, dans l'ordre d'affichage. */
export const CHOICES = [PENALTY, PENALTY_SCOPE, FIRST_PLAYER_TIE, NEXT_ROUND_STARTER, DECK, EMPTY_PILE, END_TIE] as const;

/** Texte du score de fin de partie. */
export function endScoreExplanation(endScore: number): string {
  return `La partie s'arrête à la fin de la manche où un joueur atteint ou dépasse ${endScore} points. Le total le plus bas gagne.`;
}

/** Étiquette du score de fin : « officiel » pour 100, « variante » sinon. */
export function endScoreStatus(endScore: number): Status {
  return endScore === OFFICIAL_RULES.end_score ? "officiel" : "variante";
}

export const END_SCORE_LIMITS = { min: END_SCORE_MIN, max: END_SCORE_MAX } as const;

/** Description d'une valeur de réglage (pour l'affichage en lecture seule). */
export function describe<K extends ChoiceKey>(info: ChoiceInfo<K>, value: Rules[K]): OptionInfo<Rules[K]> | undefined {
  return info.options.find((option) => option.value === value);
}

/** Raccourcis proposés pour le score de fin de partie. */
export const END_SCORE_SHORTCUTS = [50, 100, 150] as const;

/** Un préréglage : un jeu de réglages complet, choisi d'un clic. Ce n'est qu'un raccourci de l'interface ;
 * le serveur reçoit et enregistre toujours les 7 réglages. */
export interface Preset {
  id: "official" | "relaxed" | "quick";
  title: string;
  description: string;
  rules: Rules;
}

export const PRESETS: readonly Preset[] = [
  {
    id: "official",
    title: "Règles officielles",
    description: "Les règles de la notice, sans aucune variante.",
    rules: OFFICIAL_RULES,
  },
  {
    id: "relaxed",
    title: "Détente",
    description: "Pas de pénalité pour celui qui termine la manche.",
    rules: { ...OFFICIAL_RULES, penalty: "Disabled" },
  },
  {
    id: "quick",
    title: "Partie rapide",
    description: "La partie s'arrête à 50 points au lieu de 100.",
    rules: { ...OFFICIAL_RULES, end_score: 50 },
  },
];

function sameRules(a: Rules, b: Rules): boolean {
  return (Object.keys(a) as (keyof Rules)[]).every((key) => a[key] === b[key]);
}

/** Préréglage qui correspond exactement aux réglages, ou `null` (réglages personnalisés). */
export function matchPreset(rules: Rules): Preset | null {
  return PRESETS.find((preset) => sameRules(preset.rules, rules)) ?? null;
}

/** Nombre de réglages qui diffèrent des règles officielles. */
export function differencesFromOfficial(rules: Rules): number {
  return (Object.keys(OFFICIAL_RULES) as (keyof Rules)[]).filter((key) => rules[key] !== OFFICIAL_RULES[key])
    .length;
}

/** Résumé en une phrase, pour le salon. */
export function rulesSummary(rules: Rules): string {
  const preset = matchPreset(rules);
  if (preset !== null) return preset.title;
  const n = differencesFromOfficial(rules);
  return `Règles officielles, sauf ${n} réglage${n > 1 ? "s" : ""}`;
}
