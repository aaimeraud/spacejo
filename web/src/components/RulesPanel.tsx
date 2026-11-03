// Réglages de partie : des préréglages choisis d'un clic, puis une carte par réglage pour personnaliser.
// Chaque réglage montre une explication, un exemple concret et une étiquette textuelle « officiel » ou
// « variante » (jamais la couleur seule). L'interface n'affiche que les réglages reçus du serveur.
import { useEffect, useId, useState } from "react";
import type { Rules } from "../api/types";
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
  rulesSummary,
  type ChoiceInfo,
  type ChoiceKey,
  type Status,
} from "../rulesInfo";

/** « 1 réglage différent… », « 2 réglages différents… ». */
function differencesText(n: number): string {
  const plural = n > 1 ? "s" : "";
  return `${n} réglage${plural} différent${plural} des règles officielles.`;
}

function StatusTag({ status }: { status: Status }) {
  return <span className={`tag tag-${status}`}>{status}</span>;
}

// ----- Une carte de réglage -----

interface RuleCardProps<K extends ChoiceKey> {
  info: ChoiceInfo<K>;
  rules: Rules;
  /** Absent : carte en lecture seule. */
  onChange?: ((rules: Rules) => void) | undefined;
  disabled?: boolean;
}

function RuleCard<K extends ChoiceKey>({ info, rules, onChange, disabled = false }: RuleCardProps<K>) {
  const id = useId();
  const [open, setOpen] = useState(false);
  const current = describe(info, rules[info.key]);
  const editable = onChange !== undefined;
  const summary = (
    <>
      <span className="rule-icon" aria-hidden="true">
        {info.icon}
      </span>
      <span className="rule-card-text">
        <span className="rule-card-title">{info.short}</span>
        <strong className="rule-card-value">{current?.label ?? String(rules[info.key])}</strong>
      </span>
      {current !== undefined && <StatusTag status={current.status} />}
    </>
  );
  return (
    // Pointillés seulement si le réglage s'écarte des règles officielles.
    <div className={`rule-card${open ? " is-open" : ""}${rules[info.key] !== OFFICIAL_RULES[info.key] ? " is-variant" : ""}`}>
      {editable ? (
        <button
          type="button"
          className="rule-card-head"
          aria-expanded={open}
          aria-controls={`${id}-body`}
          onClick={() => setOpen(!open)}
        >
          {summary}
          <span className="rule-chevron" aria-hidden="true">
            {open ? "▴" : "▾"}
          </span>
        </button>
      ) : (
        <div className="rule-card-head">{summary}</div>
      )}
      {(open || !editable) && (
        <div id={`${id}-body`} className="rule-card-body">
          <p className="rule-question">{info.title}</p>
          {editable && (
            <fieldset className="segmented" disabled={disabled}>
              <legend className="visually-hidden">{info.title}</legend>
              {info.options.map((option) => {
                const optionId = `${id}-${option.value}`;
                return (
                  <span key={option.value} className="segment">
                    <input
                      type="radio"
                      id={optionId}
                      name={id}
                      value={option.value}
                      checked={rules[info.key] === option.value}
                      onChange={() => onChange({ ...rules, [info.key]: option.value })}
                    />
                    <label htmlFor={optionId}>
                      {option.label}
                      <StatusTag status={option.status} />
                    </label>
                  </span>
                );
              })}
            </fieldset>
          )}
          {current !== undefined && (
            <>
              <p className="explanation">{current.explanation}</p>
              <p className="example">
                <span className="example-label">Exemple :</span> {current.example}
              </p>
            </>
          )}
          {info.note !== undefined && <p className="note">{info.note}</p>}
        </div>
      )}
    </div>
  );
}

// ----- Score de fin de partie -----

function EndScoreCard({
  rules,
  onChange,
  disabled = false,
}: {
  rules: Rules;
  onChange?: ((rules: Rules) => void) | undefined;
  disabled?: boolean;
}) {
  const id = useId();
  // Valeur affichée pendant que le curseur bouge ; envoyée seulement au relâchement, pour ne pas
  // envoyer une requête par cran.
  const [draft, setDraft] = useState(rules.end_score);
  useEffect(() => setDraft(rules.end_score), [rules.end_score]);
  const commit = (value: number) => {
    if (onChange !== undefined && value !== rules.end_score) onChange({ ...rules, end_score: value });
  };
  return (
    <div className={`rule-card is-open end-score-card${draft !== OFFICIAL_RULES.end_score ? " is-variant" : ""}`}>
      <div className="rule-card-head">
        <span className="rule-icon" aria-hidden="true">
          🏁
        </span>
        <span className="rule-card-text">
          <span className="rule-card-title">Fin de partie</span>
          <strong className="rule-card-value">{draft} points</strong>
        </span>
        <StatusTag status={endScoreStatus(draft)} />
      </div>
      <div className="rule-card-body">
        {onChange !== undefined && (
          <>
            <label htmlFor={`${id}-range`} className="visually-hidden">
              Score de fin de partie ({END_SCORE_LIMITS.min} à {END_SCORE_LIMITS.max} points)
            </label>
            <input
              id={`${id}-range`}
              className="end-score-range"
              type="range"
              min={END_SCORE_LIMITS.min}
              max={END_SCORE_LIMITS.max}
              step={10}
              value={draft}
              disabled={disabled}
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
                  disabled={disabled}
                  onClick={() => commit(value)}
                >
                  {value}
                </button>
              ))}
            </div>
          </>
        )}
        <p className="explanation">{endScoreExplanation(draft)}</p>
      </div>
    </div>
  );
}

// ----- Éditeur complet (création de partie, salon de l'hôte) -----

interface EditorProps {
  rules: Rules;
  onChange: (rules: Rules) => void;
  disabled?: boolean;
}

export function RulesEditor({ rules, onChange, disabled = false }: EditorProps) {
  const preset = matchPreset(rules);
  const [customizing, setCustomizing] = useState(preset === null);
  const panelId = useId();
  const showCards = customizing || preset === null;
  return (
    <div className="rules-editor">
      <div className="preset-grid" role="group" aria-label="Type de partie">
        {PRESETS.map((p) => (
          <button
            key={p.id}
            type="button"
            className={`preset${preset?.id === p.id ? " is-selected" : ""}`}
            aria-pressed={preset?.id === p.id}
            disabled={disabled}
            onClick={() => onChange(p.rules)}
          >
            <span className="preset-title">{p.title}</span>
            <span className="preset-description">{p.description}</span>
            <StatusTag status={p.id === "official" ? "officiel" : "variante"} />
          </button>
        ))}
        <button
          type="button"
          className={`preset${preset === null ? " is-selected" : ""}`}
          aria-pressed={preset === null}
          aria-expanded={showCards}
          aria-controls={panelId}
          disabled={disabled}
          onClick={() => setCustomizing(true)}
        >
          <span className="preset-title">Personnalisée</span>
          <span className="preset-description">
            {preset === null
              ? differencesText(differencesFromOfficial(rules))
              : "Choisir chaque réglage soi-même."}
          </span>
          <StatusTag status="variante" />
        </button>
      </div>

      <button
        type="button"
        className="link-button"
        aria-expanded={showCards}
        aria-controls={panelId}
        onClick={() => setCustomizing(!showCards)}
        disabled={preset === null}
      >
        {showCards ? "Masquer le détail des règles" : "Voir et modifier le détail des règles"}
      </button>

      {showCards && (
        <div id={panelId} className="rule-cards">
          {CHOICES.map((info) => (
            <RuleCard
              key={info.key}
              info={info as ChoiceInfo<ChoiceKey>}
              rules={rules}
              onChange={onChange}
              disabled={disabled}
            />
          ))}
          <EndScoreCard rules={rules} onChange={onChange} disabled={disabled} />
        </div>
      )}
    </div>
  );
}

// ----- Lecture seule (salon, joueurs qui ne sont pas l'hôte) -----

export function RulesSummary({ rules }: { rules: Rules }) {
  return (
    <div className="rules-summary">
      <p className="rules-summary-line">
        <strong>{rulesSummary(rules)}</strong>{" "}
        <StatusTag status={differencesFromOfficial(rules) === 0 ? "officiel" : "variante"} />
      </p>
      <div className="rule-cards">
        {CHOICES.map((info) => (
          <RuleCard key={info.key} info={info as ChoiceInfo<ChoiceKey>} rules={rules} />
        ))}
        <EndScoreCard rules={rules} />
      </div>
    </div>
  );
}
