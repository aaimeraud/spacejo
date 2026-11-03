#!/bin/sh
# Prépare une release : version, changelog, commit `chore(release)` et tag annoté.
#
# Usage : scripts/release.sh [version] [--dry-run]
#   version   : par exemple 0.1.0 ou v0.1.0. Sans argument, la version suivante est calculée par
#               git-cliff d'après les commits depuis le dernier tag (`git cliff --bumped-version`).
#               La toute première release (v0.1.0) se donne en argument.
#   --dry-run : affiche la version et l'aperçu du changelog, sans rien modifier.
#
# Ce script ne pousse JAMAIS : il affiche les commandes `git push` à lancer soi-même.
# Le tag poussé déclenche .github/workflows/release.yml, qui publie la release GitHub.
set -eu

MAIN_BRANCH="main"
CHANGELOG="docs/CHANGELOG.md"

version=""
dry_run=0
for arg in "$@"; do
  case "$arg" in
    --dry-run) dry_run=1 ;;
    -h|--help) sed -n '2,12p' "$0"; exit 0 ;;
    -*) echo "Option inconnue : $arg" >&2; exit 2 ;;
    *)
      if [ -n "$version" ]; then echo "Une seule version attendue." >&2; exit 2; fi
      version="$arg" ;;
  esac
done

cd "$(git rev-parse --show-toplevel)"

command -v git-cliff >/dev/null 2>&1 || {
  echo "git-cliff est introuvable : https://git-cliff.org/docs/installation" >&2
  exit 1
}

# 1. Arbre de travail propre et branche principale.
if [ -n "$(git status --porcelain)" ]; then
  echo "Refus : l'arbre de travail n'est pas propre (git status)." >&2
  exit 1
fi
branch=$(git rev-parse --abbrev-ref HEAD)
if [ "$branch" != "$MAIN_BRANCH" ]; then
  echo "Refus : la release se fait depuis « $MAIN_BRANCH » (branche actuelle : « $branch »)." >&2
  exit 1
fi

# 2. Version : argument, ou calcul par git-cliff (section [bump] de cliff.toml).
if [ -z "$version" ]; then
  if [ -z "$(git tag --list 'v[0-9]*')" ]; then
    echo "Aucun tag de version : donnez la première version en argument (par exemple 0.1.0)." >&2
    exit 1
  fi
  version=$(git cliff --bumped-version 2>/dev/null)
fi
number=${version#v}
tag="v$number"
if ! printf '%s' "$number" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$'; then
  echo "Version invalide : « $version » (attendu : X.Y.Z)." >&2
  exit 1
fi
if git rev-parse -q --verify "refs/tags/$tag" >/dev/null; then
  echo "Refus : le tag $tag existe déjà." >&2
  exit 1
fi

echo "Version : $tag"

if [ "$dry_run" -eq 1 ]; then
  echo "Aperçu du changelog (rien n'est modifié) :"
  echo
  git cliff --unreleased --tag "$tag" --strip header 2>/dev/null
  exit 0
fi

# 3. Versions dans Cargo.toml (version commune de l'espace de travail) et web/package.json.
sed -i.bak -E "/^\[workspace\.package\]/,/^\[/ s/^version = \"[^\"]*\"/version = \"$number\"/" Cargo.toml
rm -f Cargo.toml.bak
cargo update --workspace --offline >/dev/null 2>&1 || cargo update --workspace >/dev/null
(cd web && npm version "$number" --no-git-tag-version --allow-same-version >/dev/null)

# 4. Changelog complet régénéré.
git cliff --tag "$tag" -o "$CHANGELOG" 2>/dev/null

# 5. Commit et tag annoté.
git add Cargo.toml Cargo.lock web/package.json web/package-lock.json "$CHANGELOG"
git commit -m "chore(release): $tag"
git tag -a "$tag" -m "SpaceJo $tag"

echo
echo "Release $tag prête localement. Pour la publier :"
echo "  git push origin $MAIN_BRANCH"
echo "  git push origin $tag"
