// Tutoriel pour les nouveaux joueurs : une manche fixe contre Aatish (ami automatisé), étape par étape.
// - « info » : texte et dessins ; - « play » : vous cliquez sur la case entourée ; - « bot » : Aatish joue.
// L'affichage ne garde rien en mémoire : l'état du plateau est recalculé en rejouant le scénario, donc
// revenir en arrière reste juste. Même cadre que les autres parcours de l'accueil (`WizardFrame`).
import { useEffect, useState, type ReactNode } from "react";
import { TUTORIAL_BOT_PAUSE_MS, TUTORIAL_SUCCESS_PAUSE_MS } from "../config";
import {
  TUTORIAL_STEPS,
  applyStep,
  sameTarget,
  stateBeforeStep,
  targetOf,
  type TutorialStep,
  type TutorialTarget,
  type TutorialVisual,
} from "../tutorial";
import { slotOf, type TutorialState } from "../game/tutorialGame";
import { Card3D, ValueCard } from "./Card";
import { WizardFrame } from "./wizard/RuleSteps";

interface Props {
  /** Quitte le tutoriel (retour au menu de l'accueil). */
  onClose: () => void;
  /** Termine le tutoriel en ouvrant la création d'une partie. */
  onPlay: () => void;
}

/** Texte lu par un lecteur d'écran pour une carte d'exemple (les cartes dessinées sont décoratives). */
function cardLabel(card: number | null): string {
  return card === null ? "cachée" : String(card);
}

function Visual({ visual }: { visual: TutorialVisual }) {
  if (visual.kind === "cards") {
    return (
      <figure className="tuto-visual">
        <div className="tuto-cards" role="img" aria-label={`Cartes : ${visual.cards.map(cardLabel).join(", ")}`}>
          {visual.cards.map((card, i) => (
            <ValueCard key={i} value={card} size="md" />
          ))}
        </div>
        <figcaption>{visual.caption}</figcaption>
      </figure>
    );
  }
  const label = visual.columns
    .map((column, i) => `colonne ${i + 1} : ${column.map(cardLabel).join(", ")}`)
    .join(" ; ");
  return (
    <figure className="tuto-visual">
      <div className="tuto-grid" role="img" aria-label={`Grille, ${label}`}>
        {visual.columns.map((column, c) => (
          <div key={c} className="tuto-column">
            {column.map((card, r) => (
              <ValueCard key={r} value={card} size="md" />
            ))}
          </div>
        ))}
      </div>
      <figcaption>{visual.caption}</figcaption>
    </figure>
  );
}

/** Case, pioche ou défausse du plateau. Cliquable si `onClick` est donné ; `active` l'entoure (geste attendu). */
function Spot({
  label,
  active = false,
  onClick,
  children,
}: {
  label: string;
  active?: boolean;
  onClick?: (() => void) | undefined;
  children: ReactNode;
}) {
  const className = `tuto-spot${active ? " is-target" : ""}`;
  if (onClick === undefined) {
    return (
      <span className={className} role="img" aria-label={label}>
        {children}
      </span>
    );
  }
  return (
    <button type="button" className={className} aria-label={label} onClick={onClick}>
      {children}
    </button>
  );
}

/** Plateau : Aatish en haut, la pioche, la défausse et la carte en main au milieu, vous en bas. */
function Board({
  state,
  target,
  onTarget,
}: {
  state: TutorialState;
  /** Case attendue, entourée. `null` : rien à entourer. */
  target: TutorialTarget | null;
  /** Clic d'un joueur sur le plateau. Absent : plateau en lecture seule (coups de Aatish). */
  onTarget?: ((target: TutorialTarget) => void) | undefined;
}) {
  const interactive = onTarget !== undefined;
  const isTarget = (t: TutorialTarget) => target !== null && sameTarget(t, target);
  const pick = (t: TutorialTarget) => () => onTarget?.(t);

  const topDiscard = state.discard.at(-1);
  const held = state.held;

  const renderGrid = (who: "me" | "bot", title: string) => (
    <div className="tuto-side">
      <p className="tuto-side-title">{title}</p>
      <div className="tuto-board-grid" role="group" aria-label={title}>
        {state.grids[who].map((column, c) =>
          column.length === 0 ? (
            <div key={c} className="tuto-removed">
              Colonne {c + 1} retirée
            </div>
          ) : (
            <div key={c} className="tuto-column">
              {column.map((cell, r) => {
                const t: TutorialTarget = { kind: "cell", column: c, row: r };
                const label = `${title}, colonne ${c + 1}, ligne ${r + 1} : ${cell.visible ? cell.value : "cachée"}`;
                return (
                  <Spot
                    key={r}
                    label={label}
                    active={who === "me" && isTarget(t)}
                    onClick={who === "me" && interactive ? pick(t) : undefined}
                  >
                    <Card3D slot={slotOf(cell)} size="sm" />
                  </Spot>
                );
              })}
            </div>
          ),
        )}
      </div>
    </div>
  );

  return (
    <div className="tuto-board">
      {renderGrid("bot", "Grille de Aatish")}
      <div className="tuto-table">
        <Spot
          label={`Pioche, ${state.pile.length} cartes`}
          active={isTarget({ kind: "pile" })}
          onClick={interactive ? pick({ kind: "pile" }) : undefined}
        >
          <ValueCard value={null} size="sm" />
        </Spot>
        <Spot
          label={topDiscard === undefined ? "Défausse vide" : `Défausse, dessus : ${topDiscard}`}
          active={isTarget({ kind: "discard" })}
          onClick={interactive ? pick({ kind: "discard" }) : undefined}
        >
          {topDiscard === undefined ? <span className="tuto-empty">vide</span> : <ValueCard value={topDiscard} size="sm" />}
        </Spot>
        <span className="tuto-held" role="img" aria-label={held === null ? "Aucune carte en main" : `En main : ${held.value}`}>
          <span className="tuto-held-label">En main</span>
          {held === null ? <span className="tuto-empty">aucune</span> : <ValueCard value={held.value} size="sm" />}
        </span>
      </div>
      {renderGrid("me", "Votre grille")}
    </div>
  );
}

function Paragraphs({ paragraphs }: { paragraphs: string[] }) {
  return (
    <>
      {paragraphs.map((paragraph) => (
        <p key={paragraph}>{paragraph}</p>
      ))}
    </>
  );
}

export function Tutorial({ onClose, onPlay }: Props) {
  const [index, setIndex] = useState(0);
  // Étape « play » : le bon geste a-t-il été fait ?
  const [done, setDone] = useState(false);
  // Message après un clic qui n'est pas le geste attendu.
  const [hint, setHint] = useState<string | null>(null);
  const step: TutorialStep | undefined = TUTORIAL_STEPS[index];

  // Pas besoin de cliquer « Suivant » après un coup de Aatish ou un geste réussi : on passe seul à l'étape
  // suivante après une courte pause pour lire. « Suivant » reste là pour aller plus vite. Le minuteur est
  // annulé dès que l'étape change (clic sur « Suivant » ou « Retour »), donc il ne saute jamais une étape.
  const pauseMs =
    step?.kind === "bot"
      ? TUTORIAL_BOT_PAUSE_MS
      : step?.kind === "play" && done
        ? TUTORIAL_SUCCESS_PAUSE_MS
        : null;
  useEffect(() => {
    if (pauseMs === null || index === TUTORIAL_STEPS.length - 1) return;
    const timer = window.setTimeout(() => {
      setIndex(index + 1);
      setDone(false);
      setHint(null);
    }, pauseMs);
    return () => window.clearTimeout(timer);
  }, [index, done, pauseMs]);

  // `index` reste toujours dans la liste ; ce test rassure seulement TypeScript (accès par indice).
  if (step === undefined) return null;
  const last = index === TUTORIAL_STEPS.length - 1;

  const goTo = (next: number) => {
    setIndex(next);
    setDone(false);
    setHint(null);
  };

  // État du plateau : avant le geste attendu, après le geste (une fois fait), ou après le coup de Aatish.
  const before = stateBeforeStep(TUTORIAL_STEPS, index);
  const shown: TutorialState = step.kind === "play" && !done ? before : applyStep(before, step);

  const onTarget = (clicked: TutorialTarget) => {
    if (step.kind !== "play" || done) return;
    if (sameTarget(clicked, targetOf(step.action))) {
      setDone(true);
      setHint(null);
    } else {
      setHint("Ce n'est pas la case entourée. Essayez encore.");
    }
  };

  return (
    <WizardFrame
      index={index}
      total={TUTORIAL_STEPS.length}
      title={step.title}
      onBack={() => (index === 0 ? onClose() : goTo(index - 1))}
      onNext={() => (last ? onPlay() : goTo(index + 1))}
      nextLabel={last ? "Créer une partie" : "Suivant"}
      nextDisabled={step.kind === "play" && !done}
      finish={last}
    >
      {step.kind === "info" && (
        <div className="tuto-step">
          <Paragraphs paragraphs={step.paragraphs} />
          {step.visual !== undefined && <Visual visual={step.visual} />}
        </div>
      )}
      {step.kind === "play" && (
        <div className="tuto-step">
          <p className="tuto-prompt">{done ? step.success : step.prompt}</p>
          {/* Lu par les lecteurs d'écran dès qu'un message apparaît. */}
          <p className="tuto-hint" role="status">
            {!done ? hint : null}
          </p>
          <Board state={shown} target={done ? null : targetOf(step.action)} onTarget={done ? undefined : onTarget} />
        </div>
      )}
      {step.kind === "bot" && (
        <div className="tuto-step">
          <Paragraphs paragraphs={step.narration} />
          <Board state={shown} target={null} />
        </div>
      )}
    </WizardFrame>
  );
}
