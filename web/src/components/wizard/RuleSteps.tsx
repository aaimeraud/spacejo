// Étapes du parcours de réglages, partagées par la création de partie (accueil) et la modification des
// règles par l'hôte (salon) : type de partie, puis un réglage par écran si « Personnalisée », puis un
// récapitulatif. Les préréglages ne sont que des raccourcis : on envoie toujours les 7 réglages.
import { ArrowLeft, ArrowRight, Check, Pencil } from "lucide-react";
import { useEffect, useState, type ReactNode } from "react";
import type { Rules } from "../../api/types";
import {
  CHOICES,
  END_SCORE_LIMITS,
  END_SCORE_SHORTCUTS,
  OFFICIAL_RULES,
  PRESETS,
  describe,
  differencesFromOfficial,
  endScoreExplanation,
  endScoreStatus,
  matchPreset,
  type ChoiceInfo,
  type ChoiceKey,
  type Status,
} from "../../rulesInfo";
import { PRESET_ICONS, RULE_ICONS } from "../icons";

export function StatusTag({ status }: { status: Status }) {
  return <span className={`tag tag-${status}`}>{status}</span>;
}

// ----- Cadre commun d'une étape -----

interface FrameProps {
  /** Position (0 = première étape) et nombre total d'étapes, pour la barre de progression. */
  index: number;
  total: number;
  title: string;
  subtitle?: string;
  onBack?: (() => void) | undefined;
  onNext?: (() => void) | undefined;
  nextLabel?: string;
  nextDisabled?: boolean;
  /** Dernière étape : bouton principal avec une coche plutôt qu'une flèche. */
  finish?: boolean;
  children: ReactNode;
}

export function WizardFrame({
  index,
  total,
  title,
  subtitle,
  onBack,
  onNext,
  nextLabel = "Suivant",
  nextDisabled = false,
  finish = false,
  children,
}: FrameProps) {
  return (
    <div className="wizard">
      <div className="wizard-progress" role="progressbar" aria-valuemin={1} aria-valuemax={total} aria-valuenow={index + 1}>
        {Array.from({ length: total }, (_, i) => (
          <span key={i} className={`dot${i < index ? " is-done" : ""}${i === index ? " is-current" : ""}`} />
        ))}
        <span className="visually-hidden">
          Étape {index + 1} sur {total}
        </span>
      </div>
      <h2 className="wizard-title">{title}</h2>
      {subtitle !== undefined && <p className="wizard-subtitle">{subtitle}</p>}
      <div className="wizard-body">{children}</div>
      <div className="wizard-nav">
        {onBack !== undefined ? (
          <button type="button" className="ghost-button" onClick={onBack}>
            <ArrowLeft aria-hidden="true" /> Retour
          </button>
        ) : (
          <span />
        )}
        {onNext !== undefined && (
          <button type="submit" className="primary" onClick={onNext} disabled={nextDisabled}>
            {nextLabel} {finish ? <Check aria-hidden="true" /> : <ArrowRight aria-hidden="true" />}
          </button>
        )}
      </div>
    </div>
  );
}

// ----- Liste des étapes de réglage -----

export type RuleStepKey = "preset" | ChoiceKey | "end_score" | "recap";

/** Étapes du parcours : avec « Personnalisée », un écran par réglage. */
export function ruleStepKeys(custom: boolean): RuleStepKey[] {
  const settings: RuleStepKey[] = [...CHOICES.map((info) => info.key), "end_score"];
  return ["preset", ...(custom ? settings : []), "recap"];
}

export function ruleStepTitle(key: RuleStepKey): string {
  if (key === "preset") return "Quel type de partie ?";
  if (key === "recap") return "Récapitulatif des règles";
  if (key === "end_score") return "Quand la partie s'arrête-t-elle ?";
  const info = CHOICES.find((c) => c.key === key);
  return info?.title ?? "";
}

// ----- Étape « type de partie » -----

export function PresetStep({
  rules,
  custom,
  onPreset,
  onCustom,
}: {
  rules: Rules;
  custom: boolean;
  onPreset: (rules: Rules) => void;
  onCustom: () => void;
}) {
  const preset = custom ? null : matchPreset(rules);
  const CustomIcon = PRESET_ICONS.custom;
  return (
    <div className="tile-grid" role="group" aria-label="Type de partie">
      {PRESETS.map((p) => {
        const Icon = PRESET_ICONS[p.id];
        return (
          <button
            key={p.id}
            type="button"
            className={`big-tile${preset?.id === p.id ? " is-selected" : ""}`}
            aria-pressed={preset?.id === p.id}
            onClick={() => onPreset(p.rules)}
          >
            <Icon className="tile-icon" aria-hidden="true" />
            <span className="tile-title">{p.title}</span>
            <span className="tile-description">{p.description}</span>
            <StatusTag status={p.id === "official" ? "officiel" : "variante"} />
          </button>
        );
      })}
      <button
        type="button"
        className={`big-tile${preset === null ? " is-selected" : ""}`}
        aria-pressed={preset === null}
        onClick={onCustom}
      >
        <CustomIcon className="tile-icon" aria-hidden="true" />
        <span className="tile-title">Personnalisée</span>
        <span className="tile-description">Choisir chaque réglage, un écran à la fois.</span>
        <StatusTag status="variante" />
      </button>
    </div>
  );
}

// ----- Étape « un réglage » -----

export function ChoiceStep<K extends ChoiceKey>({
  info,
  rules,
  onChange,
}: {
  info: ChoiceInfo<K>;
  rules: Rules;
  onChange: (rules: Rules) => void;
}) {
  const current = describe(info, rules[info.key]);
  return (
    <div className="choice-step">
      <div className="tile-grid" role="group" aria-label={info.title}>
        {info.options.map((option) => (
          <button
            key={option.value}
            type="button"
            className={`big-tile${rules[info.key] === option.value ? " is-selected" : ""}`}
            aria-pressed={rules[info.key] === option.value}
            onClick={() => onChange({ ...rules, [info.key]: option.value })}
          >
            <span className="tile-title">{option.label}</span>
            <span className="tile-description">{option.explanation}</span>
            <StatusTag status={option.status} />
          </button>
        ))}
      </div>
      {current !== undefined && (
        <p className="example">
          <span className="example-label">Exemple :</span> {current.example}
        </p>
      )}
      {info.note !== undefined && <p className="note">{info.note}</p>}
    </div>
  );
}

// ----- Étape « score de fin » -----

export function EndScoreStep({ rules, onChange }: { rules: Rules; onChange: (rules: Rules) => void }) {
  const [draft, setDraft] = useState(rules.end_score);
  useEffect(() => setDraft(rules.end_score), [rules.end_score]);
  const commit = (value: number) => {
    if (value !== rules.end_score) onChange({ ...rules, end_score: value });
  };
  return (
    <div className="end-score-step">
      <p className="end-score-value">
        <strong>{draft}</strong> points <StatusTag status={endScoreStatus(draft)} />
      </p>
      <label htmlFor="end-score-range" className="visually-hidden">
        Score de fin de partie ({END_SCORE_LIMITS.min} à {END_SCORE_LIMITS.max} points)
      </label>
      <input
        id="end-score-range"
        className="end-score-range"
        type="range"
        min={END_SCORE_LIMITS.min}
        max={END_SCORE_LIMITS.max}
        step={10}
        value={draft}
        onChange={(event) => setDraft(Number(event.target.value))}
        onPointerUp={() => commit(draft)}
        onKeyUp={() => commit(draft)}
        onBlur={() => commit(draft)}
      />
      <div className="end-score-shortcuts" role="group" aria-label="Raccourcis">
        {END_SCORE_SHORTCUTS.map((value) => (
          <button
            key={value}
            type="button"
            className="chip"
            aria-pressed={rules.end_score === value}
            onClick={() => commit(value)}
          >
            {value}
          </button>
        ))}
      </div>
      <p className="explanation">{endScoreExplanation(draft)}</p>
    </div>
  );
}

// ----- Récapitulatif (et résumé en lecture seule du salon) -----

export function RulesRecap({
  rules,
  onEdit,
  details = false,
}: {
  rules: Rules;
  /** Absent : lecture seule. */
  onEdit?: ((key: ChoiceKey | "end_score") => void) | undefined;
  /** Affiche aussi l'explication de chaque réglage. */
  details?: boolean;
}) {
  const preset = matchPreset(rules);
  const n = differencesFromOfficial(rules);
  return (
    <div className="recap">
      <p className="recap-summary">
        {preset !== null ? preset.title : `Personnalisée : ${n} réglage${n > 1 ? "s" : ""} différent${n > 1 ? "s" : ""} des règles officielles`}
      </p>
      <ul className="recap-list">
        {CHOICES.map((info) => {
          const choice = info as ChoiceInfo<ChoiceKey>;
          const option = describe(choice, rules[info.key]);
          const Icon = RULE_ICONS[info.key];
          return (
            <li key={info.key} className={rules[info.key] !== OFFICIAL_RULES[info.key] ? "is-variant" : ""}>
              <Icon className="recap-icon" aria-hidden="true" />
              <span className="recap-text">
                <span className="recap-title">{info.short}</span>
                <strong>{option?.label ?? String(rules[info.key])}</strong>
                {details && option !== undefined && <span className="recap-detail">{option.explanation}</span>}
              </span>
              {option !== undefined && <StatusTag status={option.status} />}
              {onEdit !== undefined && (
                <button type="button" className="icon-button" onClick={() => onEdit(info.key)} aria-label={`Modifier : ${info.short}`}>
                  <Pencil aria-hidden="true" />
                </button>
              )}
            </li>
          );
        })}
        <li className={rules.end_score !== OFFICIAL_RULES.end_score ? "is-variant" : ""}>
          <Flag />
          <span className="recap-text">
            <span className="recap-title">Fin de partie</span>
            <strong>{rules.end_score} points</strong>
            {details && <span className="recap-detail">{endScoreExplanation(rules.end_score)}</span>}
          </span>
          <StatusTag status={endScoreStatus(rules.end_score)} />
          {onEdit !== undefined && (
            <button type="button" className="icon-button" onClick={() => onEdit("end_score")} aria-label="Modifier : fin de partie">
              <Pencil aria-hidden="true" />
            </button>
          )}
        </li>
      </ul>
    </div>
  );
}

function Flag() {
  const Icon = RULE_ICONS.end_score;
  return <Icon className="recap-icon" aria-hidden="true" />;
}

// ----- Corps d'une étape de réglage, selon sa clé -----

export function RuleStepBody({
  stepKey,
  rules,
  custom,
  onRules,
  onPreset,
  onCustom,
  onEdit,
}: {
  stepKey: RuleStepKey;
  rules: Rules;
  custom: boolean;
  onRules: (rules: Rules) => void;
  onPreset: (rules: Rules) => void;
  onCustom: () => void;
  onEdit: (key: ChoiceKey | "end_score") => void;
}) {
  if (stepKey === "preset") return <PresetStep rules={rules} custom={custom} onPreset={onPreset} onCustom={onCustom} />;
  if (stepKey === "recap") return <RulesRecap rules={rules} onEdit={onEdit} />;
  if (stepKey === "end_score") return <EndScoreStep rules={rules} onChange={onRules} />;
  const info = CHOICES.find((c) => c.key === stepKey);
  return info === undefined ? null : <ChoiceStep info={info as ChoiceInfo<ChoiceKey>} rules={rules} onChange={onRules} />;
}

// ----- État du parcours de réglages -----

/** État partagé par les deux parcours : réglages en cours, mode « Personnalisée », étape courante. */
export function useRulesFlow(initial: Rules) {
  const [rules, setRules] = useState<Rules>(initial);
  const [custom, setCustom] = useState(matchPreset(initial) === null);
  const steps = ruleStepKeys(custom);
  const [position, setPosition] = useState(0);
  const stepKey = steps[Math.min(position, steps.length - 1)] ?? "preset";
  return {
    rules,
    setRules,
    custom,
    steps,
    position,
    stepKey,
    setPosition,
    /** Choix d'un préréglage : on passe directement au récapitulatif. */
    pickPreset(next: Rules) {
      setRules(next);
      setCustom(false);
      setPosition(ruleStepKeys(false).indexOf("recap"));
    },
    /** « Personnalisée » : on parcourt chaque réglage. */
    pickCustom() {
      setCustom(true);
      setPosition(1);
    },
    /** Depuis le récapitulatif : revenir sur un réglage précis. */
    edit(key: ChoiceKey | "end_score") {
      setCustom(true);
      setPosition(ruleStepKeys(true).indexOf(key));
    },
  };
}
