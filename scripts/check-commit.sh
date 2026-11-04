#!/bin/sh
# Usage : <fichiers modifiés, un par ligne, sur stdin> | scripts/check-commit.sh <fichier-message>
msg_file="$1"
header=$(head -n 1 "$msg_file")
changed=$(cat)

# Commits automatiques : pas de contrôle.
case "$header" in Merge*|Revert*|fixup!*|squash!*) exit 0 ;; esac

# 1. Format Conventional Commits.
if ! printf '%s' "$header" | grep -Eq '^(feat|fix|refactor|perf|docs|test|ci|chore|style)(\([a-z0-9-]+\))?!?: .+'; then
  echo "Message refusé : format attendu « type(scope): résumé »" >&2
  echo "Types : feat, fix, refactor, perf, docs, test, ci, chore, style." >&2
  exit 1
fi

# 2. Règle de documentation (types exemptés : docs, test, ci, chore, style).
type=$(printf '%s' "$header" | sed -E 's/^([a-z]+).*/\1/')
case "$type" in docs|test|ci|chore|style) exit 0 ;; esac

# Chemins du code source : à adapter à l'arborescence réelle, UNE seule fois, ici.
# src/ : API et serveur ; crates/ : spacejo-core ;
# web/src/ : interface ; migrations/ : schéma de la base.
if printf '%s\n' "$changed" | grep -Eq '^(src/|crates/|web/src/|migrations/)'; then
  if ! printf '%s\n' "$changed" | grep -q '^docs/' && ! grep -Eq '^Docs: n/a' "$msg_file"; then
    echo "Commit refusé : ce changement touche le code sans toucher docs/." >&2
    echo "Mets la documentation à jour, ou ajoute en pied de message : « Docs: n/a — raison »." >&2
    exit 1
  fi
fi
exit 0
