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
};

export type Status = "officiel" | "variante";

export interface OptionInfo<V extends string> {
  value: V;
  label: string;
  explanation: string;
  status: Status;
}

/** Réglages dont la valeur est une énumération. */
export type ChoiceKey = Exclude<keyof Rules, "end_score">;

export interface ChoiceInfo<K extends ChoiceKey> {
  key: K;
  title: string;
  /** Mention affichée quand la notice officielle ne tranche pas. */
  note?: string;
  options: OptionInfo<Rules[K]>[];
}

const UNSPECIFIED = "La notice officielle ne précise pas ce cas.";

export const PENALTY: ChoiceInfo<"penalty"> = {
  key: "penalty",
  title: "Pénalité du joueur qui termine la manche",
  options: [
    {
      value: "DoubleUnlessStrictlyLowest",
      label: "Doublé s'il n'est pas seul le plus bas",
      explanation:
        "Son score de manche est doublé s'il n'a pas le score strictement le plus bas (une égalité le double aussi).",
      status: "officiel",
    },
    {
      value: "DoubleOnlyIfBeaten",
      label: "Doublé seulement s'il est battu",
      explanation: "Son score est doublé seulement si un autre joueur a strictement moins ; une égalité l'épargne.",
      status: "variante",
    },
    {
      value: "Disabled",
      label: "Aucune pénalité",
      explanation: "Terminer la manche n'est jamais pénalisé.",
      status: "variante",
    },
  ],
};

export const PENALTY_SCOPE: ChoiceInfo<"penalty_scope"> = {
  key: "penalty_scope",
  title: "Ce que double la pénalité",
  note: "Les sources divergent ; le livret indique le total de la manche.",
  options: [
    {
      value: "RoundTotal",
      label: "Le total de la manche",
      explanation: "Le score de manche entier est doublé. Un total nul ou négatif n'est jamais doublé.",
      status: "officiel",
    },
    {
      value: "PositiveCardsOnly",
      label: "Les cartes positives seulement",
      explanation:
        "Les cartes positives comptent deux fois, les négatives une seule fois. Un total nul ou négatif n'est jamais doublé.",
      status: "variante",
    },
  ],
};

export const FIRST_PLAYER_TIE: ChoiceInfo<"first_player_tie"> = {
  key: "first_player_tie",
  title: "Égalité pour commencer la première manche",
  note: UNSPECIFIED,
  options: [
    {
      value: "JoinOrder",
      label: "Ordre d'arrivée",
      explanation: "Parmi les ex æquo, le premier arrivé dans la partie commence (l'hôte d'abord).",
      status: "variante",
    },
    {
      value: "Random",
      label: "Tirage au sort",
      explanation: "Le serveur tire au sort parmi les ex æquo.",
      status: "variante",
    },
  ],
};

export const NEXT_ROUND_STARTER: ChoiceInfo<"next_round_starter"> = {
  key: "next_round_starter",
  title: "Qui commence la manche suivante",
  options: [
    {
      value: "Finisher",
      label: "Celui qui a terminé",
      explanation: "Le joueur qui a terminé la manche précédente commence la suivante.",
      status: "officiel",
    },
    {
      value: "RoundWinner",
      label: "Le gagnant de la manche",
      explanation:
        "Le joueur au score de manche le plus bas (après pénalité ; en cas d'égalité, le premier arrivé) commence.",
      status: "variante",
    },
    {
      value: "Rotate",
      label: "Chacun son tour",
      explanation: "Le joueur suivant celui qui a commencé la manche précédente commence.",
      status: "variante",
    },
  ],
};

export const EMPTY_PILE: ChoiceInfo<"empty_pile"> = {
  key: "empty_pile",
  title: "Pioche épuisée",
  note: UNSPECIFIED,
  options: [
    {
      value: "ReshuffleDiscard",
      label: "Remélanger la défausse",
      explanation:
        "La défausse est remélangée pour former une nouvelle pioche ; sa carte du dessus reste visible.",
      status: "variante",
    },
    {
      value: "EndRound",
      label: "Fin de manche immédiate",
      explanation: "La manche s'arrête tout de suite et on compte, sans dernier tour et sans pénalité.",
      status: "variante",
    },
  ],
};

export const END_TIE: ChoiceInfo<"end_tie"> = {
  key: "end_tie",
  title: "Égalité en fin de partie",
  note: UNSPECIFIED,
  options: [
    {
      value: "SharedVictory",
      label: "Victoire partagée",
      explanation: "Tous les joueurs à égalité sur le total le plus bas gagnent.",
      status: "variante",
    },
    {
      value: "ExtraRound",
      label: "Manche supplémentaire",
      explanation: "On rejoue des manches tant que le total le plus bas n'est pas unique.",
      status: "variante",
    },
  ],
};

/** Tous les réglages à choix, dans l'ordre d'affichage. */
export const CHOICES = [PENALTY, PENALTY_SCOPE, FIRST_PLAYER_TIE, NEXT_ROUND_STARTER, EMPTY_PILE, END_TIE] as const;

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
