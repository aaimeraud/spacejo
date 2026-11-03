// Accueil, en parcours pas à pas : créer une partie (nom, type de partie, réglages si « Personnalisée »,
// récapitulatif), rejoindre une partie (code, nom), ou reprendre une partie mémorisée. On peut toujours
// revenir à l'étape précédente.
import { History, LogIn, Plus } from "lucide-react";
import { useState, type FormEvent } from "react";
import type { Rules } from "../api/types";
import { GAME_CODE_LENGTH, MAX_NAME_CHARS } from "../config";
import { OFFICIAL_RULES } from "../rulesInfo";
import type { KnownSession } from "../session";
import { ValueCard } from "./Card";
import { FitStage } from "./FitStage";
import { RuleStepBody, WizardFrame, ruleStepTitle, useRulesFlow } from "./wizard/RuleSteps";

interface Props {
  busy: boolean;
  /** Parties rejointes sur cet appareil, qu'on peut reprendre. */
  resumable: KnownSession[];
  onCreate: (name: string, rules: Rules) => void;
  onJoin: (code: string, name: string) => void;
  onResume: (session: KnownSession) => void;
}

type Mode = "menu" | "create" | "join";

/** Cartes montrées sur l'accueil : une de chaque famille de couleur. */
const HOME_CARDS = [-2, 0, 3, 7, 12];

export function Home({ busy, resumable, onCreate, onJoin, onResume }: Props) {
  const [mode, setMode] = useState<Mode>("menu");
  const [name, setName] = useState("");
  const [code, setCode] = useState("");
  // Création : étape du nom, puis étapes de réglage (`flow`).
  const [onNameStep, setOnNameStep] = useState(true);
  // Jonction : 0 = code, 1 = nom.
  const [joinStep, setJoinStep] = useState(0);
  const flow = useRulesFlow(OFFICIAL_RULES);

  const nameValid = name.trim().length > 0;
  const codeValid = code.trim().length === GAME_CODE_LENGTH;

  const reset = (next: Mode) => {
    setMode(next);
    setOnNameStep(true);
    setJoinStep(0);
    flow.setPosition(0);
  };

  const nameField = (
    <>
      <label htmlFor="player-name" className="field-label">
        Votre nom
      </label>
      <input
        id="player-name"
        className="big-input"
        value={name}
        maxLength={MAX_NAME_CHARS}
        autoComplete="nickname"
        autoFocus
        onChange={(event) => setName(event.target.value)}
      />
      <p className="note">De 1 à {MAX_NAME_CHARS} caractères, visible par les autres joueurs.</p>
    </>
  );

  const content = () => {
    if (mode === "menu") {
      return (
        <div className="home-menu">
          <div className="home-brand">
            <div>
              <h2>Bienvenue sur SpaceJo</h2>
              <p>Un jeu de cartes au tour par tour, de 2 à 10 joueurs.</p>
            </div>
            {/* Un éventail de cartes du jeu, de la meilleure (-2) à la pire (12) : décoratif. */}
            <div className="home-cards" aria-hidden="true">
              {HOME_CARDS.map((value) => (
                <ValueCard key={value} value={value} size="md" />
              ))}
            </div>
          </div>
          <div className="tile-grid two">
            <button type="button" className="big-tile" onClick={() => reset("create")}>
              <Plus className="tile-icon" aria-hidden="true" />
              <span className="tile-title">Créer une partie</span>
              <span className="tile-description">Choisissez les règles, puis partagez le code.</span>
            </button>
            <button type="button" className="big-tile" onClick={() => reset("join")}>
              <LogIn className="tile-icon" aria-hidden="true" />
              <span className="tile-title">Rejoindre une partie</span>
              <span className="tile-description">Avec le code donné par l'hôte.</span>
            </button>
          </div>
          {resumable.length > 0 && (
            <section className="resume" aria-labelledby="resume-title">
              <h3 id="resume-title">
                <History aria-hidden="true" /> Reprendre une partie
              </h3>
              <ul className="resume-list">
                {resumable.slice(0, 3).map((known) => (
                  <li key={known.code}>
                    <button type="button" className="chip-button" onClick={() => onResume(known)}>
                      <strong>{known.code}</strong> · {known.name}
                    </button>
                  </li>
                ))}
              </ul>
            </section>
          )}
          <p className="legal">
            SpaceJo reprend les règles du jeu de cartes Skyjo. Projet indépendant, non affilié à Magilano.
          </p>
        </div>
      );
    }

    if (mode === "join") {
      const total = 2;
      return joinStep === 0 ? (
        <WizardFrame
          index={0}
          total={total}
          title="Quel est le code de la partie ?"
          subtitle="6 caractères, donnés par l'hôte."
          onBack={() => reset("menu")}
          onNext={() => setJoinStep(1)}
          nextDisabled={!codeValid}
        >
          <label htmlFor="game-code" className="visually-hidden">
            Code de la partie
          </label>
          <input
            id="game-code"
            className="big-input code-input"
            value={code}
            maxLength={GAME_CODE_LENGTH}
            autoCapitalize="characters"
            spellCheck={false}
            autoFocus
            onChange={(event) => setCode(event.target.value.toUpperCase())}
          />
        </WizardFrame>
      ) : (
        <WizardFrame
          index={1}
          total={total}
          title="Comment vous appelez-vous ?"
          onBack={() => setJoinStep(0)}
          onNext={() => onJoin(code, name)}
          nextLabel="Rejoindre"
          nextDisabled={!nameValid || busy}
          finish
        >
          {nameField}
        </WizardFrame>
      );
    }

    // Création
    const total = 1 + flow.steps.length;
    if (onNameStep) {
      return (
        <WizardFrame
          index={0}
          total={total}
          title="Comment vous appelez-vous ?"
          subtitle="Vous serez l'hôte : vous choisirez les règles et lancerez la partie."
          onBack={() => reset("menu")}
          onNext={() => setOnNameStep(false)}
          nextDisabled={!nameValid}
        >
          {nameField}
        </WizardFrame>
      );
    }
    const isRecap = flow.stepKey === "recap";
    const goBack = () => {
      if (flow.position === 0) setOnNameStep(true);
      else if (isRecap && !flow.custom) flow.setPosition(0);
      else flow.setPosition(flow.position - 1);
    };
    const goNext = () => {
      if (isRecap) onCreate(name, flow.rules);
      else if (flow.stepKey === "preset") {
        if (flow.custom) flow.setPosition(1);
        else flow.pickPreset(flow.rules);
      } else flow.setPosition(flow.position + 1);
    };
    return (
      <WizardFrame
        index={1 + flow.position}
        total={total}
        title={ruleStepTitle(flow.stepKey)}
        onBack={goBack}
        onNext={goNext}
        nextLabel={isRecap ? "Créer la partie" : "Suivant"}
        nextDisabled={busy}
        finish={isRecap}
      >
        <RuleStepBody
          stepKey={flow.stepKey}
          rules={flow.rules}
          custom={flow.custom}
          onRules={flow.setRules}
          onPreset={flow.pickPreset}
          onCustom={flow.pickCustom}
          onEdit={flow.edit}
        />
      </WizardFrame>
    );
  };

  // Un formulaire : la touche Entrée valide l'étape (le bouton « Suivant » est de type submit).
  const submit = (event: FormEvent) => event.preventDefault();
  return (
    <FitStage landscape={[960, 680]} portrait={[440, 900]}>
      {() => (
        <form className="home-card" onSubmit={submit}>
          {content()}
        </form>
      )}
    </FitStage>
  );
}
