#!/bin/sh
# Tests de scripts/release.sh --dry-run et de la règle de version de cliff.toml (critère 18).
# Chaque cas travaille dans un dépôt Git temporaire : le vrai dépôt n'est jamais touché.
set -u
root=$(cd "$(dirname "$0")/../.." && pwd)
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
failures=0

fail() { echo "FAIL - $1"; failures=$((failures + 1)); }
ok() { echo "ok   - $1"; }

# Dépôt de test : branche main, scripts et configuration copiés, quelques commits.
new_repo() {
  dir="$tmp/$1"
  mkdir -p "$dir/scripts" "$dir/docs" "$dir/web"
  cp "$root/scripts/release.sh" "$dir/scripts/"
  cp "$root/cliff.toml" "$dir/"
  printf '[workspace.package]\nversion = "0.0.0"\n' > "$dir/Cargo.toml"
  printf '{ "name": "t", "version": "0.0.0" }\n' > "$dir/web/package.json"
  git -C "$dir" init -q -b main
  git -C "$dir" config user.email test@example.com
  git -C "$dir" config user.name Test
  git -C "$dir" config commit.gpgsign false
  git -C "$dir" config tag.gpgsign false
  git -C "$dir" add -A
  git -C "$dir" commit -q -m "chore: initial commit"
  echo "$dir"
}

commit() { # commit <dépôt> <message>
  echo "$2" >> "$1/notes.txt"
  git -C "$1" add notes.txt
  git -C "$1" commit -q -m "$2"
}

# 1. Sans tag et sans argument : refus clair (la première version se donne en argument).
repo=$(new_repo first)
commit "$repo" "feat(core): first feature"
if (cd "$repo" && sh scripts/release.sh --dry-run >/dev/null 2>&1); then
  fail "sans tag ni argument, le script doit refuser"
else
  ok "sans tag ni argument : refus"
fi

# 2. Première release donnée en argument, en dry-run : version et aperçu, rien de modifié.
before=$(git -C "$repo" rev-parse HEAD)
out=$(cd "$repo" && sh scripts/release.sh 0.1.0 --dry-run 2>&1)
if printf '%s' "$out" | grep -q "Version : v0.1.0" && printf '%s' "$out" | grep -q "First feature"; then
  ok "dry-run v0.1.0 : version et aperçu du changelog"
else
  fail "dry-run v0.1.0 : sortie inattendue : $out"
fi
if [ "$(git -C "$repo" rev-parse HEAD)" = "$before" ] && [ -z "$(git -C "$repo" status --porcelain)" ] \
  && [ -z "$(git -C "$repo" tag)" ] && [ ! -f "$repo/docs/CHANGELOG.md" ]; then
  ok "dry-run : aucun commit, aucun tag, aucun fichier modifié"
else
  fail "dry-run : le dépôt a été modifié"
fi

# 3. Version suivante calculée par git-cliff après un tag.
git -C "$repo" tag -a v0.1.0 -m v0.1.0
commit "$repo" "fix(api): small fix"
out=$(cd "$repo" && sh scripts/release.sh --dry-run 2>&1)
printf '%s' "$out" | grep -q "Version : v0.1.1" && ok "fix → v0.1.1" || fail "fix → v0.1.1 attendu : $out"
commit "$repo" "feat(web): new screen"
out=$(cd "$repo" && sh scripts/release.sh --dry-run 2>&1)
printf '%s' "$out" | grep -q "Version : v0.2.0" && ok "feat → v0.2.0" || fail "feat → v0.2.0 attendu : $out"

# 4. Changement cassant en 0.x : version MINEURE (et non 1.0.0).
commit "$repo" "feat(api)!: rename routes"
out=$(cd "$repo" && sh scripts/release.sh --dry-run 2>&1)
printf '%s' "$out" | grep -q "Version : v0.2.0" && ok "feat! en 0.x → v0.2.0 (pas 1.0.0)" \
  || fail "feat! en 0.x → v0.2.0 attendu : $out"
# Preuve du réglage : avec breaking_always_bump_major = true, git-cliff passerait à 1.0.0.
sed 's/^breaking_always_bump_major = false/breaking_always_bump_major = true/' "$repo/cliff.toml" > "$tmp/major.toml"
bumped=$(cd "$repo" && git cliff --config "$tmp/major.toml" --bumped-version 2>/dev/null)
[ "$bumped" = "v1.0.0" ] && ok "réglage inverse → v1.0.0 (le réglage est utile)" \
  || fail "réglage inverse : v1.0.0 attendu, obtenu $bumped"

# 5. Refus : arbre de travail sale, ou autre branche que main.
echo "brouillon" > "$repo/sale.txt"
if (cd "$repo" && sh scripts/release.sh --dry-run >/dev/null 2>&1); then fail "arbre sale accepté"; else ok "arbre sale : refus"; fi
rm "$repo/sale.txt"
git -C "$repo" checkout -q -b autre
if (cd "$repo" && sh scripts/release.sh --dry-run >/dev/null 2>&1); then fail "autre branche acceptée"; else ok "autre branche que main : refus"; fi

if [ "$failures" -ne 0 ]; then
  echo "$failures cas en échec" >&2
  exit 1
fi
echo "Tous les cas passent."
