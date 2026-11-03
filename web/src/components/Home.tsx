// Écran d'accueil : créer une partie (en choisissant les réglages) ou en rejoindre une.
import { useState, type FormEvent } from "react";
import type { Rules } from "../api/types";
import { GAME_CODE_LENGTH, MAX_NAME_CHARS } from "../config";
import { OFFICIAL_RULES } from "../rulesInfo";
import type { KnownSession } from "../session";
import { RulesEditor } from "./RulesPanel";

interface Props {
  busy: boolean;
  /** Parties rejointes sur cet appareil, qu'on peut reprendre. */
  resumable: KnownSession[];
  onCreate: (name: string, rules: Rules) => void;
  onJoin: (code: string, name: string) => void;
  onResume: (session: KnownSession) => void;
}

export function Home({ busy, resumable, onCreate, onJoin, onResume }: Props) {
  const [createName, setCreateName] = useState("");
  const [rules, setRules] = useState<Rules>(OFFICIAL_RULES);
  const [joinName, setJoinName] = useState("");
  const [code, setCode] = useState("");

  const submitCreate = (event: FormEvent) => {
    event.preventDefault();
    onCreate(createName, rules);
  };
  const submitJoin = (event: FormEvent) => {
    event.preventDefault();
    onJoin(code, joinName);
  };

  return (
    <div className="home">
      {resumable.length > 0 && (
        <section aria-labelledby="resume-title" className="panel resume-panel">
          <h2 id="resume-title">Reprendre une partie</h2>
          <ul className="resume-list">
            {resumable.map((known) => (
              <li key={known.code}>
                <button type="button" className="preset" onClick={() => onResume(known)}>
                  <span className="preset-title">Partie {known.code}</span>
                  <span className="preset-description">En tant que {known.name}</span>
                </button>
              </li>
            ))}
          </ul>
        </section>
      )}
      <section aria-labelledby="create-title" className="panel">
        <h2 id="create-title">Créer une partie</h2>
        <form onSubmit={submitCreate}>
          <label htmlFor="create-name">Votre nom</label>
          <input
            id="create-name"
            value={createName}
            maxLength={MAX_NAME_CHARS}
            required
            autoComplete="nickname"
            onChange={(event) => setCreateName(event.target.value)}
          />
          <h3 className="subheading">Type de partie</h3>
          <RulesEditor rules={rules} onChange={setRules} disabled={busy} />
          <button type="submit" className="primary" disabled={busy}>
            Créer la partie
          </button>
        </form>
      </section>

      <section aria-labelledby="join-title" className="panel">
        <h2 id="join-title">Rejoindre une partie</h2>
        <form onSubmit={submitJoin}>
          <label htmlFor="join-code">Code de la partie</label>
          <input
            id="join-code"
            value={code}
            maxLength={GAME_CODE_LENGTH}
            required
            autoCapitalize="characters"
            spellCheck={false}
            className="code-input"
            onChange={(event) => setCode(event.target.value.toUpperCase())}
          />
          <label htmlFor="join-name">Votre nom</label>
          <input
            id="join-name"
            value={joinName}
            maxLength={MAX_NAME_CHARS}
            required
            autoComplete="nickname"
            onChange={(event) => setJoinName(event.target.value)}
          />
          <button type="submit" className="primary" disabled={busy}>
            Rejoindre
          </button>
        </form>
      </section>
    </div>
  );
}
