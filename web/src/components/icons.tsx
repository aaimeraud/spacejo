// Icônes (bibliothèque Lucide, licence ISC) associées aux réglages et aux préréglages. Elles sont
// décoratives : le texte à côté porte toujours l'information.
import {
  Coffee,
  Dices,
  Equal,
  Flag,
  Layers,
  SquareStack,
  Rocket,
  RotateCw,
  Scale,
  SlidersHorizontal,
  Sparkles,
  Zap,
  type LucideIcon,
} from "lucide-react";
import type { ChoiceKey, Preset } from "../rulesInfo";

export const RULE_ICONS: Record<ChoiceKey | "end_score", LucideIcon> = {
  penalty: Zap,
  penalty_scope: Scale,
  first_player_tie: Dices,
  next_round_starter: RotateCw,
  empty_pile: Layers,
  deck: SquareStack,
  end_tie: Equal,
  end_score: Flag,
};

export const PRESET_ICONS: Record<Preset["id"] | "custom", LucideIcon> = {
  official: Sparkles,
  relaxed: Coffee,
  quick: Rocket,
  custom: SlidersHorizontal,
};
