#!/bin/sh
# Aperçu des changements non publiés (depuis le dernier tag), dans le terminal.
# N'écrit aucun fichier.
set -eu
cd "$(git rev-parse --show-toplevel)"
git cliff --unreleased --strip header 2>/dev/null
