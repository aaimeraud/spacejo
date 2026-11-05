// Chat de la partie : un bouton (avec le nombre de messages non lus) et un panneau posé sur le côté de l'écran
// (en bas sur un téléphone). Le panneau ne bloque pas le jeu : on peut jouer pendant qu'il est ouvert.
// Chaque message des autres est précédé de l'initiale de son auteur (l'« avatar ») ; sous chaque bulle, les
// réactions (une par joueur), un bouton « Réagir », et un double tap / double clic sur la bulle pour le cœur.
import { MessageCircle, Send, SmilePlus, X } from "lucide-react";
import { useEffect, useId, useRef, useState, type FormEvent, type PointerEvent } from "react";
import type { ChatMessage, ReactionName } from "../api/types";
import { CHAT_REACTIONS, MAX_CHAT_CHARS, QUICK_REACTION } from "../config";
import { initial, isDoubleTap, lastMessageId, myReaction, nextReaction, reactionGroups } from "../game/chat";

/** Bouton d'ouverture du chat. Le nombre de non-lus est écrit en chiffres (pas seulement une pastille de couleur). */
export function ChatButton({ unread, onClick }: { unread: number; onClick: () => void }) {
  const label = unread > 0 ? `Chat, ${unread} message${unread > 1 ? "s" : ""} non lu${unread > 1 ? "s" : ""}` : "Chat";
  return (
    <button type="button" className="ghost-button chat-button" onClick={onClick} aria-label={label}>
      <MessageCircle aria-hidden="true" />
      <span className="stage-action-label">Chat</span>
      {unread > 0 && (
        <span className="chat-unread" aria-hidden="true">
          {unread > 99 ? "99+" : unread}
        </span>
      )}
    </button>
  );
}

interface PanelProps {
  messages: ChatMessage[];
  /** Mon identifiant de joueur : mes messages sont alignés à droite. */
  me: number;
  /** Nom actuel d'un joueur (pour dire qui a réagi). */
  nameOf: (player: number) => string;
  /** Pose, change ou retire (`null`) ma réaction à un message. */
  onReact: (message: number, reaction: ReactionName | null) => void;
  /** Envoie un message ; renvoie `true` s'il est parti (le champ est alors vidé). */
  onSend: (text: string) => Promise<boolean>;
  onClose: () => void;
}

export function ChatPanel({ messages, me, nameOf, onSend, onReact, onClose }: PanelProps) {
  const [draft, setDraft] = useState("");
  const [sending, setSending] = useState(false);
  // Message dont le sélecteur de réactions est ouvert (un seul à la fois).
  const [picking, setPicking] = useState<number | null>(null);
  const pickingRef = useRef<number | null>(null);
  pickingRef.current = picking;
  // Dernier appui sur une bulle (message et instant), pour reconnaître un double tap.
  const lastTap = useRef<{ id: number; time: number } | null>(null);
  const titleId = useId();
  const list = useRef<HTMLOListElement>(null);
  const input = useRef<HTMLInputElement>(null);

  // Le champ reçoit le focus à l'ouverture. Échap ferme le sélecteur de réactions s'il est ouvert, sinon le panneau.
  useEffect(() => {
    input.current?.focus();
    const onKey = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      if (pickingRef.current !== null) setPicking(null);
      else onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  // Toujours montrer le dernier message quand un nouveau arrive (pas quand seules les réactions changent : on ne
  // perd pas de vue le message ancien auquel on vient de réagir).
  const newest = lastMessageId(messages);
  useEffect(() => {
    const element = list.current;
    if (element !== null) element.scrollTop = element.scrollHeight;
  }, [newest]);

  /** Réagit avec `name` au message : la même réaction que la mienne la retire, une autre la remplace. */
  const choose = (message: ChatMessage, name: ReactionName) => {
    onReact(message.id, nextReaction(message, me, name));
    setPicking(null);
  };

  /** Appui sur une bulle : au deuxième appui rapproché (double tap ou double clic), le cœur. */
  const tap = (message: ChatMessage, event: PointerEvent<HTMLDivElement>) => {
    if (event.pointerType === "mouse" && event.button !== 0) return;
    const now = event.timeStamp;
    const previous = lastTap.current?.id === message.id ? lastTap.current.time : null;
    if (isDoubleTap(previous, now)) {
      lastTap.current = null;
      choose(message, QUICK_REACTION);
    } else {
      lastTap.current = { id: message.id, time: now };
    }
  };

  /** Liste lisible des joueurs (« Alice et Bob », « vous, Alice et Bob »). */
  const who = (players: number[]): string => {
    const names = players.map((player) => (player === me ? "vous" : nameOf(player)));
    return names.length <= 1 ? (names[0] ?? "") : `${names.slice(0, -1).join(", ")} et ${names.at(-1) ?? ""}`;
  };

  const text = draft.trim();
  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (text.length === 0 || sending) return;
    setSending(true);
    try {
      if (await onSend(text)) setDraft("");
    } finally {
      setSending(false);
      input.current?.focus();
    }
  };

  return (
    <aside className="chat-panel" aria-labelledby={titleId}>
      <header className="chat-head">
        <h2 id={titleId}>Chat de la partie</h2>
        <button type="button" className="icon-button" onClick={onClose} aria-label="Fermer le chat">
          <X aria-hidden="true" />
        </button>
      </header>
      {/* `role="log"` : les lecteurs d'écran annoncent les nouveaux messages, sans couper ce qu'ils lisent. */}
      <ol className="chat-messages" ref={list} role="log" aria-live="polite" aria-label="Messages">
        {messages.length === 0 && <li className="note chat-empty">Aucun message pour l'instant. Dites bonjour !</li>}
        {messages.map((message) => {
          const mine = message.player === me;
          const author = mine ? "Vous" : message.name;
          const groups = reactionGroups(message, me);
          const current = myReaction(message, me);
          return (
            <li key={message.id} className={`chat-message${mine ? " is-mine" : ""}`}>
              {!mine && (
                <span className="chat-avatar" aria-hidden="true">
                  {initial(message.name)}
                </span>
              )}
              <div className="chat-body">
                {/* La bulle n'est pas un bouton : le double tap est un raccourci, le bouton « Réagir » fait la même chose
                    au clavier et pour les lecteurs d'écran. */}
                <div className="chat-bubble" onPointerUp={(event) => tap(message, event)}>
                  <span className="chat-author">{author}</span>
                  <span className="chat-text">{message.text}</span>
                </div>
                <div className="chat-reactions">
                  {groups.map((group) => (
                    <button
                      key={group.name}
                      type="button"
                      className={`reaction-chip${group.mine ? " is-mine" : ""}`}
                      aria-pressed={group.mine}
                      aria-label={`${group.label} : ${who(group.players)}`}
                      title={who(group.players)}
                      onClick={() => choose(message, group.name)}
                    >
                      <span className="reaction-emoji" aria-hidden="true">{group.emoji}</span>
                      {group.players.length > 1 && <span className="reaction-count">{group.players.length}</span>}
                    </button>
                  ))}
                  <button
                    type="button"
                    className="reaction-add"
                    aria-expanded={picking === message.id}
                    aria-label={mine ? "Réagir à votre message" : `Réagir au message de ${author}`}
                    onClick={() => setPicking((open) => (open === message.id ? null : message.id))}
                  >
                    <SmilePlus aria-hidden="true" />
                  </button>
                </div>
                {picking === message.id && (
                  <div className="reaction-picker" role="group" aria-label="Choisir une réaction">
                    {CHAT_REACTIONS.map((reaction) => (
                      <button
                        key={reaction.name}
                        type="button"
                        className={`reaction-option${current === reaction.name ? " is-mine" : ""}`}
                        aria-pressed={current === reaction.name}
                        aria-label={reaction.label}
                        onClick={() => choose(message, reaction.name)}
                      >
                        <span className="reaction-emoji" aria-hidden="true">{reaction.emoji}</span>
                      </button>
                    ))}
                  </div>
                )}
              </div>
            </li>
          );
        })}
      </ol>
      <form className="chat-form" onSubmit={(event) => void submit(event)}>
        <label htmlFor={`${titleId}-input`} className="visually-hidden">
          Votre message
        </label>
        <input
          id={`${titleId}-input`}
          ref={input}
          className="chat-input"
          value={draft}
          maxLength={MAX_CHAT_CHARS}
          autoComplete="off"
          placeholder="Écrire un message…"
          onChange={(event) => setDraft(event.target.value)}
        />
        <button type="submit" className="primary" disabled={text.length === 0 || sending} aria-label="Envoyer">
          <Send aria-hidden="true" />
        </button>
      </form>
    </aside>
  );
}
