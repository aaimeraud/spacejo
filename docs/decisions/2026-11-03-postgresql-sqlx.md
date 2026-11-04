# PostgreSQL avec sqlx, verrou optimiste et migrations embarquées

## Contexte

Les parties doivent survivre entre deux requêtes (fonctions sans état), être protégées contre les
écritures concurrentes, et le schéma ne doit évoluer que par des migrations versionnées.

## Décision

- Driver **`sqlx` 0.9** (fonctions `sqlx::query`, sans les macros vérifiées à la compilation), TLS par
  `rustls` + `ring` (aucune bibliothèque système à installer), migrations par `sqlx::migrate!`, qui embarque
  `migrations/` dans les binaires.
- Le serveur applique les migrations à son démarrage, en local comme en production (voir la décision
  « serveur unique sur Render ») ; `cargo run --bin migrate` les applique sans lancer le serveur.
- Le stockage est un `enum Store { Memory, Postgres }` plutôt qu'un trait : plus simple à lire, et la
  variante mémoire sert aux tests et au développement sans base.
- Verrou optimiste : `UPDATE ... WHERE id = $1 AND version = $2`, et refus (409) si `expected_version`
  ne correspond pas avant même d'appliquer l'action.

## Alternatives écartées

- `tokio-postgres` + `refinery` + une caisse TLS : trois dépendances au lieu d'une.
- Verrou pessimiste (`SELECT ... FOR UPDATE` dans une transaction) : demande une transaction ouverte
  pendant le calcul, et ne protège pas contre un écran périmé (d'où `expected_version` de toute façon).
- Appliquer les migrations à chaque requête : ralentit chaque appel et donne à toutes les fonctions le
  droit de modifier le schéma.

## Conséquences

- Une requête rejouée ou envoyée depuis un écran périmé est refusée sans rien écrire.
- Les tests PostgreSQL (`tests/postgres.rs`) ont besoin de `DATABASE_URL` ; la CI fournit un service
  PostgreSQL et rend ces tests obligatoires (`SPACEJO_REQUIRE_DB=1`).
