// Chat de la partie : un bouton (avec le nombre de messages non lus) et un panneau posé sur le côté de l'écran
// (en bas sur un téléphone). Le panneau ne bloque pas le jeu : on peut jouer pendant qu'il est ouvert.
import { MessageCircle, Send, X } from "lucide-react";
import { useEffect, useId, useRef, useState, type FormEvent } from "react";
import type { ChatMessage } from "../api/types";
import { MAX_CHAT_CHARS } from "../config";

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
  /** Envoie un message ; renvoie `true` s'il est parti (le champ est alors vidé). */
  onSend: (text: string) => Promise<boolean>;
  onClose: () => void;
}

export function ChatPanel({ messages, me, onSend, onClose }: PanelProps) {
  const [draft, setDraft] = useState("");
  const [sending, setSending] = useState(false);
  const titleId = useId();
  const list = useRef<HTMLOListElement>(null);
  const input = useRef<HTMLInputElement>(null);

  // Le champ reçoit le focus à l'ouverture ; Échap ferme le panneau.
  useEffect(() => {
    input.current?.focus();
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  // Toujours montrer le dernier message.
  useEffect(() => {
    const element = list.current;
    if (element !== null) element.scrollTop = element.scrollHeight;
  }, [messages]);

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
        {messages.map((message) => (
          <li key={message.id} className={`chat-message${message.player === me ? " is-mine" : ""}`}>
            <span className="chat-author">{message.player === me ? "Vous" : message.name}</span>
            <span className="chat-text">{message.text}</span>
          </li>
        ))}
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
