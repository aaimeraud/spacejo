// Salon : code à partager, joueurs (jusqu'à 10), règles actives. L'hôte modifie les règles avec le même
// parcours pas à pas qu'à la création (dans une fenêtre), puis lance la partie. Tout tient sur un écran.
import { BookOpen, Copy, Crown, Pencil, Play, Users, WifiOff } from "lucide-react";
import { useState } from "react";
import type { GameView, PresenceEntry, Rules } from "../api/types";
import { MAX_PLAYERS } from "../config";
import { FitStage } from "./FitStage";
import { Modal } from "./Modal";
import { RuleStepBody, RulesRecap, WizardFrame, ruleStepTitle, useRulesFlow } from "./wizard/RuleSteps";

interface Props {
  view: GameView;
  code: string;
  busy: boolean;
  presence: PresenceEntry[];
  onRulesChange: (rules: Rules) => void;
  onStart: () => void;
}

/** Parcours de modification des règles (hôte), dans une fenêtre. */
function RulesWizardModal({
  initial,
  onSave,
  onClose,
}: {
  initial: Rules;
  onSave: (rules: Rules) => void;
  onClose: () => void;
}) {
  const flow = useRulesFlow(initial);
  const isRecap = flow.stepKey === "recap";
  const goBack =
    flow.position === 0
      ? undefined
      : () => (isRecap && !flow.custom ? flow.setPosition(0) : flow.setPosition(flow.position - 1));
  const goNext = () => {
    if (isRecap) {
      onSave(flow.rules);
      onClose();
    } else if (flow.stepKey === "preset") {
      if (flow.custom) flow.setPosition(1);
      else flow.pickPreset(flow.rules);
    } else flow.setPosition(flow.position + 1);
  };
  return (
    <Modal title="Modifier les règles" onClose={onClose} wide>
      <WizardFrame
        index={flow.position}
        total={flow.steps.length}
        title={ruleStepTitle(flow.stepKey)}
        onBack={goBack}
        onNext={goNext}
        nextLabel={isRecap ? "Enregistrer" : "Suivant"}
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
    </Modal>
  );
}

export function Lobby({ view, code, busy, presence, onRulesChange, onStart }: Props) {
  const [editing, setEditing] = useState(false);
  const [explaining, setExplaining] = useState(false);
  const [copied, setCopied] = useState(false);
  const connected = (id: number) => presence.find((p) => p.player === id)?.connected ?? true;
  const isHost = view.me === view.host;
  const enoughPlayers = view.players.length >= 2;

  return (
    <>
      <FitStage landscape={[1180, 700]} portrait={[440, 960]}>
        {() => (
          <div className="lobby">
            <section className="panel lobby-players" aria-labelledby="players-title">
              <p className="field-label">Code de la partie</p>
              <div className="code-row">
                <p className="game-code" aria-label={`Code : ${code.split("").join(" ")}`}>
                  {code}
                </p>
                <button
                  type="button"
                  className="icon-button"
                  aria-label="Copier le code"
                  onClick={() => {
                    void navigator.clipboard?.writeText(code).then(() => setCopied(true));
                  }}
                >
                  <Copy aria-hidden="true" />
                </button>
              </div>
              <p className="note" role="status">
                {copied ? "Code copié !" : "Partagez ce code avec les autres joueurs."}
              </p>
              <h2 id="players-title">
                <Users aria-hidden="true" /> Joueurs ({view.players.length} / {MAX_PLAYERS})
              </h2>
              <ul className="player-chips">
                {view.players.map((player) => (
                  <li key={player.id} className={`player-chip${connected(player.id) ? "" : " is-absent"}`}>
                    <span className="badge-avatar" aria-hidden="true">
                      {player.name.slice(0, 1).toUpperCase()}
                    </span>
                    <span className="player-chip-name">
                      {player.name}
                      {player.id === view.me && " (vous)"}
                    </span>
                    {player.id === view.host && (
                      <span className="pill pill-host">
                        <Crown aria-hidden="true" /> hôte
                      </span>
                    )}
                    {!connected(player.id) && (
                      <span className="pill pill-absent">
                        <WifiOff aria-hidden="true" /> absent
                      </span>
                    )}
                  </li>
                ))}
              </ul>
              <div className="lobby-start">
                {isHost ? (
                  <>
                    <button type="button" className="primary big" onClick={onStart} disabled={busy || !enoughPlayers}>
                      <Play aria-hidden="true" /> Lancer la partie
                    </button>
                    {!enoughPlayers && <p className="note">Il faut au moins 2 joueurs.</p>}
                  </>
                ) : (
                  <p className="note">En attente du lancement par l'hôte.</p>
                )}
              </div>
            </section>

            <section className="panel lobby-rules" aria-labelledby="rules-title">
              <h2 id="rules-title">Règles de la partie</h2>
              <RulesRecap rules={view.rules} />
              <div className="lobby-rules-actions">
                {isHost && (
                  <button type="button" onClick={() => setEditing(true)} disabled={busy}>
                    <Pencil aria-hidden="true" /> Modifier les règles
                  </button>
                )}
                <button type="button" className="ghost-button" onClick={() => setExplaining(true)}>
                  <BookOpen aria-hidden="true" /> Explications
                </button>
              </div>
            </section>
          </div>
        )}
      </FitStage>
      {editing && <RulesWizardModal initial={view.rules} onSave={onRulesChange} onClose={() => setEditing(false)} />}
      {explaining && (
        <Modal title="Règles actives, en détail" onClose={() => setExplaining(false)} wide>
          <RulesRecap rules={view.rules} details />
        </Modal>
      )}
    </>
  );
}
