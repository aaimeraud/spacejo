// Tutoriel pour les nouveaux joueurs : les étapes de `tutorial.ts`, une à la fois, avec des cartes d'exemple.
// Même cadre que les autres parcours de l'accueil (barre de progression, « Retour », « Suivant »).
import { useState } from "react";
import { TUTORIAL_STEPS, type TutorialVisual } from "../tutorial";
import { ValueCard } from "./Card";
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
          <div key={c} className={`tuto-column${c === visual.highlightColumn ? " is-highlighted" : ""}`}>
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

export function Tutorial({ onClose, onPlay }: Props) {
  const [index, setIndex] = useState(0);
  const step = TUTORIAL_STEPS[index];
  const last = index === TUTORIAL_STEPS.length - 1;
  // `index` reste toujours dans la liste ; ce test rassure seulement TypeScript (accès par indice).
  if (step === undefined) return null;
  return (
    <WizardFrame
      index={index}
      total={TUTORIAL_STEPS.length}
      title={step.title}
      onBack={() => (index === 0 ? onClose() : setIndex(index - 1))}
      onNext={() => (last ? onPlay() : setIndex(index + 1))}
      nextLabel={last ? "Créer une partie" : "Suivant"}
      finish={last}
    >
      <div className="tuto-step">
        {step.paragraphs.map((paragraph) => (
          <p key={paragraph}>{paragraph}</p>
        ))}
        {step.visual !== undefined && <Visual visual={step.visual} />}
      </div>
    </WizardFrame>
  );
}
