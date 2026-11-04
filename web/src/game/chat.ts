// Messages du chat côté navigateur : fusion des messages reçus (sans doublon, dans l'ordre) et nombre de messages
// non lus. Le serveur reste la référence : il numérote les messages.
import type { ChatMessage } from "../api/types";
import { MAX_CHAT_KEPT } from "../config";

/** Ajoute `incoming` à `existing` : sans doublon (même numéro), du plus ancien au plus récent, en ne gardant que
 * les `MAX_CHAT_KEPT` derniers. Un même message peut arriver deux fois (réponse à l'envoi, puis relecture). */
export function mergeMessages(existing: ChatMessage[], incoming: ChatMessage[]): ChatMessage[] {
  const byId = new Map<number, ChatMessage>();
  for (const message of [...existing, ...incoming]) byId.set(message.id, message);
  return [...byId.values()].sort((a, b) => a.id - b.id).slice(-MAX_CHAT_KEPT);
}

/** Numéro du dernier message connu (0 s'il n'y en a pas), pour ne relire que les suivants. */
export function lastMessageId(messages: ChatMessage[]): number {
  return messages.at(-1)?.id ?? 0;
}

/** Messages des autres joueurs arrivés après le dernier message vu (`seenId`). Les siens ne comptent pas. */
export function unreadCount(messages: ChatMessage[], seenId: number, me: number): number {
  return messages.filter((message) => message.id > seenId && message.player !== me).length;
}
