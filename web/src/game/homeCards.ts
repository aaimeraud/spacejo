// Éventail décoratif de l'accueil : des valeurs tirées au hasard à chaque visite, pour que les cartes (et donc leurs
// couleurs, qui suivent la valeur) changent d'une fois sur l'autre. Purement décoratif : aucun rapport avec le
// mélange du paquet, fait par le serveur.
import { CARD_MAX_VALUE, CARD_MIN_VALUE, HOME_CARD_COUNT } from "../config";

/** `count` valeurs de cartes différentes, tirées au hasard, de la plus petite à la plus grande (l'éventail va de la
 * meilleure carte à la pire). `random` renvoie un nombre dans [0, 1[ ; remplaçable dans les tests. */
export function randomHomeCards(random: () => number = Math.random, count = HOME_CARD_COUNT): number[] {
  const values: number[] = [];
  for (let value = CARD_MIN_VALUE; value <= CARD_MAX_VALUE; value += 1) values.push(value);
  // Mélange de Fisher-Yates partiel : les `count` premières places reçoivent des valeurs tirées sans remise.
  const picked = Math.min(count, values.length);
  for (let i = 0; i < picked; i += 1) {
    const j = i + Math.floor(random() * (values.length - i));
    [values[i], values[j]] = [values[j] ?? 0, values[i] ?? 0];
  }
  return values.slice(0, picked).sort((a, b) => a - b);
}
