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
17. **Préréglages** : « Détente » (pénalité désactivée) et « Partie rapide » (fin à 50 points) sont des raccourcis de
    l'interface, composés uniquement de valeurs existantes de `Rules` ; ils n'ajoutent aucune règle. Une combinaison
    qui ne correspond à aucun préréglage est affichée comme « Personnalisée ».

## Absence et élimination (demandées par le propriétaire du dépôt)

18. **Absent** : un joueur est absent quand le serveur n'a reçu aucune requête de sa part depuis
    `PRESENCE_TIMEOUT_SECS` (15 s ; l'interface interroge le serveur toutes les 2 s).
19. **Attente d'un tour** : quand c'est au tour d'un joueur absent, on l'attend `ABSENT_TURN_WAIT_SECS` (45 s, la
    durée retenue pour « un tour ») depuis la dernière modification de la partie. Passé ce délai, la requête suivante
    d'un autre joueur passe son tour : une carte qu'il tenait en main va sur la défausse. Pendant la mise en place,
    passer son tour retourne à sa place les cartes qu'il lui restait à retourner (les premières cartes cachées).
    L'hôte peut passer ce tour sans attendre (`POST /api/skip`) : c'est alors un tour manqué comme un autre.
20. **Élimination** : au deuxième tour manqué d'affilée (`MAX_MISSED_TURNS` = 2), le joueur est éliminé. Jouer une
    action remet son compteur à zéro. Un joueur éliminé garde ses cartes de la manche en cours sur la table, mais il
    ne joue plus, ne reçoit plus de cartes, et n'est plus compté pour la pénalité, la fin de partie, le choix du
    premier joueur ni le classement. S'il ne reste qu'un joueur, la partie s'arrête et il gagne.
21. **Derniers tours** : un joueur absent pendant les derniers tours d'une manche perd son dernier tour comme
    n'importe quel autre tour ; la manche se termine quand le tour revient au joueur qui l'a terminée. (Un
    finisseur ne peut pas être éliminé dans la même manche : on n'élimine un joueur que pendant son tour.)
22. **Nombre de joueurs** : de 2 à 10 (`MAX_PLAYERS`). À 10 joueurs, 121 cartes sont distribuées et la pioche n'en
    compte que 29 : le réglage `empty_pile` intervient donc plus souvent.


## Parties publiques et lien d'invitation (demandés par le propriétaire du dépôt)

23. **Visibilité par défaut** : une partie est privée sauf si l'hôte la déclare publique à la création ; les
    parties créées avant cette évolution sont privées. L'hôte peut la changer au salon, jamais après le lancement.
24. **Partie listée** : une partie publique n'apparaît dans la liste que si elle est au salon, pas pleine, et que
    son hôte est présent (même délai que l'absence, `PRESENCE_TIMEOUT_SECS`). Un salon dont l'hôte est parti
    disparaît de la liste, et y revient si l'hôte revient.
25. **Lien d'invitation** : il ne porte que le code de la partie (`/?code=…`). Ouvert sur un appareil qui a déjà
    rejoint cette partie, il la reprend au lieu de la rejoindre une seconde fois.

## Quitter, exclure, limitation de débit (demandés par le propriétaire du dépôt)

26. **Quitter le salon** : le joueur est retiré et les suivants avancent d'une place. Si l'hôte part, le joueur arrivé
    juste après lui devient l'hôte (avec ses droits : réglages, visibilité, lancement). Le dernier joueur d'un salon
    n'en est pas retiré (le salon reste, sans hôte présent : il disparaît de la liste des parties publiques).
27. **Abandon** : quitter une partie lancée, c'est l'abandonner. Le joueur est éliminé tout de suite, avec les
    conséquences d'une élimination pour absence (n° 20), et marqué « a abandonné » pour tous. Il ne peut pas revenir.
    Quitter une partie finie ne change rien.
28. **Départ sans version attendue** : `POST /api/leave` n'envoie pas d'`expected_version` (comme la jonction, n° 16) :
    un départ ne doit pas échouer parce que la partie vient de changer ; le serveur relit et réessaie.
29. **Limites de débit** : 10 créations, 30 jonctions et 60 lectures de la liste des parties publiques par adresse IP
    et par minute. Plusieurs personnes peuvent partager une adresse (maison, lycée) : les limites restent larges.
    L'adresse vient de l'en-tête `CF-Connecting-IP` (Cloudflare, devant Render), qu'un navigateur ne peut pas imposer.
30. **Retournements rejoués** : un retournement de mise en place refusé pour conflit de version est rejoué jusqu'à
    3 fois (`MAX_REVEAL_RETRIES`), après relecture de l'état, seulement si la carte peut encore être retournée.

## Chat (demandé par le propriétaire du dépôt)

31. **Qui écrit** : tout joueur qui a encore son jeton peut écrire dans le chat, à n'importe quelle phase, y compris
    un joueur éliminé pour absence (il peut regarder la suite de la partie). L'interface ne montre le chat que sur
    l'écran de partie et de fin de partie (pas au salon).
32. **Format** : un message tient sur une ligne, de 1 à 200 caractères, sans mise en forme. L'historique affiché à
    l'ouverture est limité aux 50 derniers messages. À la reprise d'une partie, les messages des autres joueurs de
    cet historique comptent comme non lus.
33. **Pas de modération** : aucun filtre de mots, aucun signalement ; la limitation de débit (30 messages par minute
    et par adresse) empêche seulement d'inonder le chat.

## Pioche réduite (demandée par le propriétaire du dépôt)

34. **Pioche réduite** (`deck` = `Reduced`) : 40 cartes par joueur encore en jeu (12 pour sa grille, 28 pour la pioche),
    au plus les 150 cartes (donc sans effet à partir de 4 joueurs). Le premier choix, 20, laissait une pioche de
    15 cartes à 2 joueurs, jugée trop courte par le propriétaire du dépôt. Ce nombre est une constante (`REDUCED_DECK_CARDS_PER_PLAYER`) ; la demande disait seulement
    « moins de cartes, proportionnellement au nombre de joueurs ». Les cartes gardées sont les premières du paquet
    mélangé : un tirage au hasard, qui respecte la proportion des valeurs en moyenne mais pas exactement. Le paquet est
    refait à chaque manche, avec le nombre de joueurs encore en jeu (un joueur éliminé ne compte plus).
35. **Animation du remélange** : l'interface la déduit de deux vues successives (la pioche grossit pendant une même
    manche), sans information de plus du serveur. Un remélange survenu pendant qu'un navigateur était fermé n'est
    pas rejoué.
