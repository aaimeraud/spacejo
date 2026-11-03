# Générateur aléatoire du core : SplitMix64 écrit à la main

## Contexte

Le mélange du paquet doit être fait par le serveur, avec un générateur initialisable par une graine,
pour que les tests soient déterministes (CLAUDE.md, règle 8). Les fonctions serverless ne gardent rien en
mémoire : l'état du générateur doit être sauvegardé avec la partie pour que les tirages suivants
(remélange de la défausse, tirage au sort du premier joueur) continuent la même suite.

## Décision

`crates/spacejo-core/src/rng.rs` contient un générateur SplitMix64 (une vingtaine de lignes) et un
mélange de Fisher-Yates. Son état (un `u64`) est un champ de `Game`, donc enregistré dans le JSON de la
partie. Il n'apparaît jamais dans la vue filtrée, et son `Debug` est écrit à la main pour ne pas l'afficher.
En production, l'API tire la graine de l'entropie du système (`getrandom`).

## Alternatives écartées

- **Caisse `rand` + `rand_chacha`** : de bonne qualité, mais une mise à jour peut changer l'ordre produit
  pour une même graine (les tests déterministes casseraient), et l'état de ChaCha est plus lourd à
  sérialiser. Le core resterait aussi dépendant de caisses externes pour une fonction simple.
- **Ne sauvegarder que la graine et un compteur** : possible, mais plus difficile à expliquer que
  « l'état du générateur est enregistré tel quel ».

## Conséquences

- Les tests (critères 1, 3, 13) sont reproductibles indéfiniment.
- SplitMix64 n'est pas cryptographique : connaître son état permettrait de prédire les cartes. L'état reste
  donc côté serveur (base de données), comme les cartes cachées elles-mêmes. Les jetons et les codes de
  partie, eux, utilisent l'entropie du système, pas ce générateur.
