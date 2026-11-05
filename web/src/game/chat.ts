// Messages du chat côté navigateur : fusion des messages reçus (sans doublon, dans l'ordre) et nombre de messages
// non lus. Le serveur reste la référence : il numérote les messages.
import type { ChatMessage, ReactionName } from "../api/types";
import { CHAT_REACTIONS, DOUBLE_TAP_MS, MAX_CHAT_KEPT } from "../config";

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

/** Initiale d'un nom pour son avatar : le premier caractère, en majuscule (`Array.from` garde un emoji entier). */
export function initial(name: string): string {
  return (Array.from(name.trim())[0] ?? "?").toLocaleUpperCase("fr");
}

/** Une réaction d'un message, regroupée : combien de joueurs l'ont mise, lesquels, et si j'en fais partie. */
export interface ReactionGroup {
  name: ReactionName;
  emoji: string;
  label: string;
  players: number[];
  mine: boolean;
}

/** Réactions d'un message regroupées, dans l'ordre du sélecteur (`CHAT_REACTIONS`), sans les réactions absentes. */
export function reactionGroups(message: ChatMessage, me: number): ReactionGroup[] {
  return CHAT_REACTIONS.map(({ name, emoji, label }) => {
    const players = message.reactions.filter((r) => r.reaction === name).map((r) => r.player);
    return { name, emoji, label, players, mine: players.includes(me) };
  }).filter((group) => group.players.length > 0);
}

/** Ma réaction à un message, s'il y en a une. */
export function myReaction(message: ChatMessage, me: number): ReactionName | null {
  return message.reactions.find((r) => r.player === me)?.reaction ?? null;
}

/** Réaction à envoyer quand je choisis `name` : la même que la mienne la retire (`null`), une autre la remplace. */
export function nextReaction(message: ChatMessage, me: number, name: ReactionName): ReactionName | null {
  return myReaction(message, me) === name ? null : name;
}

/** Un appui à `now` forme-t-il un double tap avec le précédent (`previous`, ou `null` s'il n'y en a pas) ? */
export function isDoubleTap(previous: number | null, now: number): boolean {
  return previous !== null && now - previous >= 0 && now - previous < DOUBLE_TAP_MS;
}
