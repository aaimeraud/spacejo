# Contrôle des commits : hook local, CI, et GitButler

## Contexte

Chaque commit doit respecter Conventional Commits et la règle de documentation
(`scripts/check-commit.sh`). Le contrôle tourne dans le hook `.githooks/commit-msg` et dans la CI.
Le dépôt est manipulé avec GitButler (`but`), dont la commande `but commit` ne lance pas le hook
`commit-msg` de Git (vérifié avec `but` 0.23.2 sur un dépôt de test).

## Décision

- Le hook reste en place pour qui commite avec `git commit` (`git config core.hooksPath .githooks`).
- Avec GitButler, on lance le même script à la main avant chaque commit, sur la liste des fichiers du
  commit : `git status --porcelain -- <fichiers> | ... | scripts/check-commit.sh <message>`.
- La CI (job `commit-checks`) reste le contrôle de référence, sur chaque commit d'une pull request et sur
  son titre.

## Alternatives écartées

- Abandonner le hook : il reste utile avec `git commit`.
- Contourner le contrôle : interdit (CLAUDE.md).

## Conséquences

Un commit fait avec GitButler sans lancer le script à la main n'est attrapé que par la CI.
