#!/bin/sh
# Tests de scripts/check-commit.sh (critère d'acceptation 18).
# Usage : tests/scripts/check-commit.test.sh (depuis la racine du dépôt).
# Chaque cas fournit un message et une liste de fichiers, et vérifie le code de sortie.
set -u
script="$(dirname "$0")/../../scripts/check-commit.sh"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
failures=0

# check <nom> <code attendu> <message> <fichiers séparés par des espaces>
check() {
  name="$1"; expected="$2"; message="$3"; files="$4"
  printf '%s\n' "$message" > "$tmp/msg"
  printf '%s\n' $files | sh "$script" "$tmp/msg" 2>/dev/null
  actual=$?
  if [ "$actual" -eq "$expected" ]; then
    echo "ok   - $name"
  else
    echo "FAIL - $name (attendu $expected, obtenu $actual)"
    failures=$((failures + 1))
  fi
}

check "message invalide refusé" 1 "added stuff" "README.md"
check "type inconnu refusé" 1 "feature(core): add deck" "crates/spacejo-core/src/lib.rs"
check "code sans docs refusé" 1 "feat(core): add deck" "crates/spacejo-core/src/lib.rs"
check "code de l'API sans docs refusé" 1 "fix(api): handle conflict" "src/handlers.rs"
check "code avec docs accepté" 0 "feat(core): add deck" "crates/spacejo-core/src/lib.rs docs/ARCHITECTURE.md"
check "Docs: n/a accepté" 0 "$(printf 'refactor(core): rename helper\n\nDocs: n/a — renommage interne')" "crates/spacejo-core/src/lib.rs"
check "commit de type docs accepté" 0 "docs: explain the lock" "docs/ARCHITECTURE.md"
check "commit de type test exempté" 0 "test(core): cover deck" "crates/spacejo-core/tests/deck.rs"
check "commit de fusion accepté" 0 "Merge branch 'main' into feature" "src/lib.rs"
check "changement cassant accepté" 0 "feat(api)!: rename endpoint" "api/create.rs docs/CONTRACT.md"
check "feat hors code accepté sans docs" 0 "feat: add vercel config" "vercel.json"

if [ "$failures" -ne 0 ]; then
  echo "$failures cas en échec" >&2
  exit 1
fi
echo "Tous les cas passent."
