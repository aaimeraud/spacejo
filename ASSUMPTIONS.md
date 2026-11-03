# Hypothèses

Cas que `docs/RULES.md` ne couvre pas, ou couvre de façon ambiguë, et le choix retenu. Chaque
hypothèse est appliquée par le code indiqué et, quand c'est possible, vérifiée par un test.

## Règles du jeu

1. **Pioche vide et défausse d'une seule carte** (imposée par la demande) : quel que soit `empty_pile`,
   la manche se termine comme pour `EndRound` (tout est retourné et compté, sans dernier tour).
   Code : `Game::end_move` (`crates/spacejo-core/src/game.rs`). Test : `empty_pile.rs`.
2. **Moment où la pioche est « épuisée »** : on le constate à la fin du coup qui a pris la dernière carte
   (et non quand le joueur suivant voudrait piocher). Ainsi, la pioche n'est jamais vide au début d'un
   tour, et `EndRound` termine bien la manche « immédiatement ».
3. **`EndRound` alors qu'un joueur a déjà terminé** : la règle dit « personne n'a terminé, donc pas de
   pénalité ». Si un joueur avait déjà retourné toutes ses cartes avant l'épuisement de la pioche, il
   reste le finisseur : sa pénalité s'applique, et les derniers tours restants ne sont pas joués.
   Test : `end_round_after_someone_finished_still_penalizes_the_finisher`.
4. **`Finisher` sans finisseur** (manche terminée par la pioche épuisée) : la manche suivante est
   commencée selon la règle de la première manche (plus grande somme des 2 cartes retournées, égalité
   départagée par `first_player_tie`). Test : `without_a_finisher_the_finisher_rule_falls_back_to_the_highest_sum`.
5. **Colonnes identiques au retournement final** : la règle « 3 cartes visibles de même valeur → colonne
   retirée » est appliquée à la lettre, y compris quand toutes les cartes sont retournées en fin de
   manche. Cela ne change le score que si la colonne ne vaut pas 0. (Idée : en faire un réglage,
   voir `IDEAS.md`.)
6. **Mise en place sans ordre de tour** : pendant la phase où chacun retourne ses 2 cartes, les joueurs
   jouent dans n'importe quel ordre. La partie commence quand tous ont fini. Ce retournement a lieu à
   chaque manche, pas seulement à la première.
7. **Premier joueur des manches suivantes** : la règle « plus grande somme » ne vaut que pour la première
   manche ; ensuite c'est `next_round_starter` qui décide (après la mise en place de la manche).
8. **Carte piochée** : tant qu'elle est en main, seule la personne qui l'a piochée en voit la valeur (les
   autres voient qu'une carte est en main). Elle devient publique dès qu'elle est posée ou défaussée.
   Une carte prise sur la défausse était déjà visible : sa valeur reste publique.
9. **Manche suivante** : n'importe quel joueur peut lancer la distribution de la manche suivante
   (bouton « Manche suivante »), pour qu'un hôte absent ne bloque pas la partie.
10. **Nouvelle manche** : on repart d'un paquet complet de 150 cartes mélangé (équivalent à ramasser
    toutes les cartes).
11. **Défausse quand une colonne est retirée** : la carte remplacée est posée d'abord, puis les 3 cartes
    de la colonne ; la carte du dessus de la défausse est donc une carte de la colonne retirée.
12. **Défausser une carte piochée** n'est possible que s'il reste une carte face cachée à retourner. Ce cas
    ne se présente pas en jeu normal (un joueur sans carte cachée a terminé la manche et ne rejoue pas).

## Interface et API

13. **Étiquettes « officiel » / « variante »** : pour les réglages dont la valeur par défaut est officielle
    (`penalty`, `next_round_starter`, `end_score`, et `penalty_scope` d'après la transcription du
    livret), la valeur par défaut est étiquetée « officiel » et les autres « variante ». Pour les réglages
    que la notice ne précise pas (`first_player_tie`, `empty_pile`, `end_tie`), toutes les valeurs sont
    étiquetées « variante », avec la mention « la notice ne précise pas ce cas ».
14. **Nom de joueur** : 1 à 20 caractères après suppression des espaces autour ; deux joueurs peuvent
    avoir le même nom (ils sont distingués par leur place).
15. **Code de partie** : la saisie est mise en majuscules et débarrassée des espaces autour.
16. **Jonction** : rejoindre n'envoie pas d'`expected_version` (le joueur ne connaît pas encore la partie) ;
    en cas d'écriture concurrente, le serveur relit la partie et réessaie quelques fois.
