// Rejouer une écriture refusée pour conflit de version (409). Utile pour les retournements de la mise en place :
// tous les joueurs retournent leurs cartes en même temps, et chaque retournement change la version de la partie.

export interface RetryOptions {
  /** Nombre maximal de nouveaux essais après le premier. */
  retries: number;
  isConflict: (error: unknown) => boolean;
  /** Appelée après un conflit : relit l'état et dit si l'écriture a encore un sens (sinon, on abandonne). */
  beforeRetry: () => Promise<boolean>;
}

/** Exécute `attempt`, et le rejoue après chaque conflit tant que c'est permis. Les autres erreurs, et le dernier
 * conflit, remontent à l'appelant. Renvoie `null` si `beforeRetry` a dit que l'écriture n'avait plus de sens. */
export async function retryOnConflict<T>(attempt: () => Promise<T>, options: RetryOptions): Promise<T | null> {
  for (let tries = 0; ; tries += 1) {
    try {
      return await attempt();
    } catch (error) {
      if (!options.isConflict(error) || tries >= options.retries) throw error;
      if (!(await options.beforeRetry())) return null;
    }
  }
}
