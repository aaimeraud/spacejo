# Règles du jeu

> Source de vérité des règles. Le code de `crates/spacejo-core` les applique ; toute hypothèse sur un cas
> non couvert est notée dans `ASSUMPTIONS.md` à la racine du dépôt.

SpaceJo reprend les règles du jeu de cartes Skyjo (éditeur : Magilano), avec les réglages décrits plus bas.

Statuts, vérifiés le 2026-11-03 sur les sources listées en fin de section :
- **[officiel]** : confirmé par la notice officielle ou par plusieurs transcriptions concordantes.
- **[option]** : la notice officielle ne précise pas ce cas, ou les sources divergent. Ce n'est donc PAS une
  règle fixe : c'est un réglage choisi par l'hôte à la création de la partie (voir « Règles maison »).

Les règles sans étiquette n'étaient pas en doute.

- **Cartes** : 150 cartes. 5 × (-2), 10 × (-1), 15 × 0, et 10 × chacune des valeurs de 1 à 12 [officiel].
  Variante : pioche réduite, proportionnelle au nombre de joueurs, réglage `deck` [option].
  (Le total de 150 est confirmé ; la répartition exacte n'a pas été relue dans les sources consultées.)
- **Mise en place** : chaque joueur reçoit 12 cartes face cachée, en grille de 3 lignes × 4 colonnes.
  Une carte est retournée au centre pour démarrer la défausse, le reste forme la pioche.
  Chaque joueur retourne 2 de ses cartes.
  - Le joueur dont ces 2 cartes ont la plus grande somme commence la première manche [officiel].
  - En cas d'égalité sur cette somme : réglage `first_player_tie` [option].
- **Un tour** : le joueur choisit l'une de ces deux voies.
  - Piocher la carte du dessus de la pioche, puis soit l'échanger contre une carte de sa grille
    (face cachée ou visible), soit la défausser et retourner une de ses cartes face cachée.
  - Prendre la carte visible du dessus de la défausse : il doit alors l'échanger contre une carte
    de sa grille (face cachée ou visible).
  - La carte remplacée est posée face visible sur la défausse (même si elle était cachée).
- **Colonne identique** : si les 3 cartes d'une colonne sont visibles et de même valeur, la colonne
  est retirée du jeu de ce joueur et ses 3 cartes vont sur la défausse.
- **Fin de manche** : dès qu'un joueur a toutes ses cartes visibles (les colonnes retirées ne comptent
  plus), chacun des autres joueurs joue un dernier tour. Ensuite toutes les cartes restantes sont
  retournées et la manche est comptée : score de manche = somme des cartes restantes d'un joueur.
- **Pénalité** : si le joueur qui a terminé la manche n'a pas le score de manche strictement le plus bas,
  son score est doublé.
  - Une égalité avec un autre joueur ne le sauve pas : il est doublé [officiel].
  - Un total nul ou négatif n'est jamais doublé [officiel, d'après deux transcriptions].
  - Le doublement porte sur le total de la manche [officiel d'après la transcription du livret ;
    d'autres sources parlent des seules cartes positives : voir `penalty_scope`].
  - Variantes (égalité épargnée, pénalité désactivée) : réglage `penalty` [option].
- **Manche suivante** : le score de manche est ajouté au total de chaque joueur, puis on redistribue.
  Dans SpaceJo, chaque joueur vote pour jouer la manche suivante et l'hôte la lance avec les votants ; les autres
  quittent la partie [choix de l'application, voir `ASSUMPTIONS.md` n° 9].
  Le joueur qui a terminé la manche précédente commence la suivante [officiel, d'après deux transcriptions ;
  une source divergente]. Autres variantes : réglage `next_round_starter` [option].
- **Fin de partie** : la partie s'arrête à la fin de la manche où un joueur atteint ou dépasse
  `end_score` (100 par défaut) [officiel]. Le total le plus bas gagne [officiel].
  En cas d'égalité sur le total le plus bas : réglage `end_tie` [option].
- **Pioche épuisée** : la notice officielle n'en parle pas. Comportement choisi par le réglage `empty_pile`
  [option].
- Si une règle est ambiguë ou absente, n'invente rien : écris ton hypothèse dans `ASSUMPTIONS.md`.

Sources consultées : notice Magilano en allemand relayée par
[ludovico.at](https://www.ludovico.at/wp-content/uploads/2025/12/Skyjo-Anleitung.pdf) ; transcription du
livret sur [rulespal.com](https://rulespal.com/skyjo/rulebook) ; aperçu non officiel sur
[spielstrategen.de](https://spielstrategen.de/spiele/skyjo/regeln/) ; résumé de la
[Province de Liège](https://www.provincedeliege.be/sites/default/files/media/15553/Skyjo.pdf).
Le texte original de Magilano n'a pas pu être lu directement. Écarts constatés : la notice ludovico dit
« plus de 100 points » pour la fin de partie alors que les autres sources disent « 100 ou plus » (retenu :
atteint ou dépasse) ; une source dit que le gagnant de la manche commence la suivante, deux autres que c'est
celui qui l'a terminée (retenu : celui qui l'a terminée).

## Règles maison (réglages de partie)

L'hôte choisit les réglages à la création de la partie, et peut les modifier au salon tant que la partie
n'est pas lancée. Sans choix de sa part, toutes les valeurs par défaut s'appliquent : ce sont les règles
officielles. Toute règle marquée [option] ci-dessus est un réglage, jamais une valeur en dur dans le code.
Ces réglages sont la structure `Rules` de `crates/spacejo-core/src/rules.rs`.

| Réglage | Valeurs (défaut en premier) | Statut |
|---|---|---|
| `penalty` | `DoubleUnlessStrictlyLowest`, `DoubleOnlyIfBeaten`, `Disabled` | défaut officiel, les deux autres sont des variantes |
| `penalty_scope` | `RoundTotal`, `PositiveCardsOnly` | sources ambiguës |
| `first_player_tie` | `JoinOrder`, `Random` | non précisé par la notice |
| `next_round_starter` | `Finisher`, `RoundWinner`, `Rotate` | défaut officiel (deux transcriptions), les autres sont des variantes |
| `deck` | `Full`, `Reduced` | défaut officiel ; `Reduced` est une variante demandée par le propriétaire du dépôt |
| `empty_pile` | `ReshuffleDiscard`, `EndRound` | non précisé par la notice |
| `end_tie` | `SharedVictory`, `ExtraRound` | non précisé par la notice |
| `end_score` | 100 (entier de 10 à 500) | défaut officiel |

Sens de chaque valeur :
- `DoubleUnlessStrictlyLowest` : doublé si le finisseur n'a pas le score strictement le plus bas
  (une égalité entraîne le doublement). `DoubleOnlyIfBeaten` : doublé seulement si un autre joueur a
  strictement moins (une égalité épargne). `Disabled` : aucune pénalité.
- `RoundTotal` : on double le total de la manche. `PositiveCardsOnly` : on double seulement la valeur des
  cartes positives, les cartes négatives comptent une seule fois. Dans les deux cas, un total nul ou
  négatif n'est jamais doublé.
- `JoinOrder` : parmi les ex æquo, le premier arrivé dans la partie commence (l'hôte d'abord).
  `Random` : tirage au sort par le serveur, avec la graine de la partie.
- `Finisher` : celui qui a terminé la manche commence la suivante. `RoundWinner` : celui qui a le score de
  manche le plus bas (après pénalité ; en cas d'égalité, le premier dans l'ordre d'arrivée). `Rotate` : le
  joueur suivant dans l'ordre de jeu après celui qui a commencé la manche précédente.
- `Full` : chaque manche se joue avec les 150 cartes. `Reduced` : au début de chaque manche, le paquet est
  mélangé puis on n'en garde que 40 cartes par joueur encore en jeu (`REDUCED_DECK_CARDS_PER_PLAYER`), au plus 150 :
  12 pour la grille de chacun, 1 pour la défausse, le reste pour la pioche (55 cartes à 2 joueurs, 83 à 3 ; à partir de 4 joueurs, le
  plafond de 150 est atteint et la partie se joue avec le paquet entier). Les
  cartes gardées sont tirées au hasard, donc la proportion des valeurs reste en moyenne celle du paquet complet.
  La pioche s'épuise vite : avec `ReshuffleDiscard`, la défausse redevient souvent la pioche ; avec `EndRound`,
  les manches sont plus courtes.
- `ReshuffleDiscard` : on remélange la défausse, sauf sa carte du dessus qui reste visible, pour former la
  nouvelle pioche. `EndRound` : la manche se termine immédiatement, tous retournent leurs cartes et on
  compte, sans dernier tour ; comme personne n'a « terminé », il n'y a pas de pénalité. Dans les deux cas,
  si la pioche est vide et que la défausse n'a qu'une carte, la manche se termine comme pour `EndRound`
  (hypothèse notée dans `ASSUMPTIONS.md`).
- `SharedVictory` : tous les ex æquo gagnent. `ExtraRound` : on joue une manche supplémentaire avec tous les
  joueurs, et on recommence tant que le total le plus bas n'est pas unique.

Règles d'usage des réglages :
- Seul l'hôte (créateur de la partie) peut changer les réglages, et seulement avant le lancement.
  Après le lancement, toute tentative est refusée. Une valeur inconnue, ou un `end_score` hors de 10 à 500,
  est refusé.
- Le serveur enregistre les réglages complets dans l'état de la partie (pas seulement ceux que l'hôte a
  modifiés), pour que la partie reste identique même si les valeurs par défaut changent plus tard.
- Tous les joueurs voient les règles actives dans le salon avant le début de la partie.
- L'interface affiche les règles reçues du serveur ; elle ne recalcule jamais un score elle-même.
