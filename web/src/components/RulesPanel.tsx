// Affichage et modification des réglages de partie. Chaque réglage montre une phrase d'explication
// et une étiquette textuelle « officiel » ou « variante » (jamais la couleur seule).
import { useEffect, useId, useState } from "react";
import type { Rules } from "../api/types";
import {
  CHOICES,
  END_SCORE_LIMITS,
  OFFICIAL_RULES,
  describe,
  endScoreExplanation,
  endScoreStatus,
  type ChoiceInfo,
  type ChoiceKey,
  type Status,
} from "../rulesInfo";

function StatusTag({ status }: { status: Status }) {
  return <span className={`tag tag-${status}`}>{status}</span>;
}

/** Réglages en lecture seule (salon, pour les joueurs qui ne sont pas l'hôte). */
export function RulesSummary({ rules }: { rules: Rules }) {
  return (
    <dl className="rules-summary">
      {CHOICES.map((info) => {
        const option = describe(info as ChoiceInfo<ChoiceKey>, rules[info.key]);
        return (
          <div key={info.key} className="rule-row">
            <dt>{info.title}</dt>
            <dd>
              <strong>{option?.label ?? rules[info.key]}</strong>{" "}
              {option !== undefined && <StatusTag status={option.status} />}
              <p>{option?.explanation}</p>
              {info.note !== undefined && <p className="note">{info.note}</p>}
            </dd>
          </div>
        );
      })}
      <div className="rule-row">
        <dt>Score de fin de partie</dt>
        <dd>
          <strong>{rules.end_score} points</strong> <StatusTag status={endScoreStatus(rules.end_score)} />
          <p>{endScoreExplanation(rules.end_score)}</p>
        </dd>
      </div>
    </dl>
  );
}

interface EditorProps {
  rules: Rules;
  onChange: (rules: Rules) => void;
  disabled?: boolean;
}

/** Réglages modifiables (création de partie, et salon pour l'hôte). */
export function RulesEditor({ rules, onChange, disabled = false }: EditorProps) {
  const baseId = useId();
  // Le score de fin est saisi librement puis envoyé à la sortie du champ, pour ne pas envoyer
  // une valeur intermédiaire (« 1 », « 15 »…) à chaque frappe.
  const [endScoreText, setEndScoreText] = useState(String(rules.end_score));
  useEffect(() => setEndScoreText(String(rules.end_score)), [rules.end_score]);
  const commitEndScore = () => {
    const value = Number.parseInt(endScoreText, 10);
    if (Number.isNaN(value)) {
      setEndScoreText(String(rules.end_score));
    } else if (value !== rules.end_score) {
      onChange({ ...rules, end_score: value });
    }
  };
  return (
    <div className="rules-editor">
      {CHOICES.map((info) => {
        const choice = info as ChoiceInfo<ChoiceKey>;
        const selected = describe(choice, rules[info.key]);
        return (
          <fieldset key={info.key} className="rule-row" disabled={disabled}>
            <legend>{info.title}</legend>
            {info.note !== undefined && <p className="note">{info.note}</p>}
            {choice.options.map((option) => {
              const id = `${baseId}-${info.key}-${option.value}`;
              return (
                <div key={option.value} className="option">
                  <input
                    type="radio"
                    id={id}
                    name={`${baseId}-${info.key}`}
                    value={option.value}
                    checked={rules[info.key] === option.value}
                    onChange={() => onChange({ ...rules, [info.key]: option.value })}
                  />
                  <label htmlFor={id}>
                    {option.label} <StatusTag status={option.status} />
                  </label>
                </div>
              );
            })}
            {selected !== undefined && <p className="explanation">{selected.explanation}</p>}
          </fieldset>
        );
      })}
      <fieldset className="rule-row" disabled={disabled}>
        <legend>Score de fin de partie</legend>
        <label htmlFor={`${baseId}-end-score`}>
          Points ({END_SCORE_LIMITS.min} à {END_SCORE_LIMITS.max}){" "}
          <StatusTag status={endScoreStatus(rules.end_score)} />
        </label>
        <input
          id={`${baseId}-end-score`}
          type="number"
          min={END_SCORE_LIMITS.min}
          max={END_SCORE_LIMITS.max}
          step={10}
          value={endScoreText}
          onChange={(event) => setEndScoreText(event.target.value)}
          onBlur={commitEndScore}
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              event.preventDefault();
              commitEndScore();
            }
          }}
        />
        <p className="explanation">{endScoreExplanation(rules.end_score)}</p>
      </fieldset>
      <button type="button" onClick={() => onChange(OFFICIAL_RULES)} disabled={disabled}>
        Règles officielles
      </button>
    </div>
  );
}
