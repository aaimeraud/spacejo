# Notions Rust rencontrées

Chaque ligne : la notion, le fichier où elle apparaît, et le chapitre du livre
[The Rust Programming Language](https://doc.rust-lang.org/stable/book/).

| Notion | Fichier | Chapitre |
|---|---|---|
| Espace de travail Cargo (plusieurs paquets, un seul `Cargo.lock`) | `Cargo.toml` | [ch14-03](https://doc.rust-lang.org/stable/book/ch14-03-cargo-workspaces.html) |
| Modules et `pub use` | `crates/spacejo-core/src/lib.rs` | [ch07-02](https://doc.rust-lang.org/stable/book/ch07-02-defining-modules-to-control-scope-and-privacy.html) |
| Enum avec données (`Column::Present(Vec<Cell>)`, `Phase::Playing(Turn)`) | `crates/spacejo-core/src/grid.rs`, `game.rs` | [ch06-01](https://doc.rust-lang.org/stable/book/ch06-01-defining-an-enum.html) |
| `match` et motifs (couples, gardes `if`) | `crates/spacejo-core/src/scoring.rs`, `game.rs` | [ch06-02](https://doc.rust-lang.org/stable/book/ch06-02-match.html) |
| `Option` et `Result`, opérateur `?`, `ok_or` | `crates/spacejo-core/src/grid.rs`, `game.rs` | [ch09-02](https://doc.rust-lang.org/stable/book/ch09-02-recoverable-errors-with-result.html) |
| Emprunts `&` et `&mut` (lecture seule ou modification) | `crates/spacejo-core/src/rng.rs`, `view.rs` | [ch04-02](https://doc.rust-lang.org/stable/book/ch04-02-references-and-borrowing.html) |
| Traits (`Default`, `Debug` écrit à la main, `Display`) | `crates/spacejo-core/src/rules.rs`, `rng.rs`, `error.rs` | [ch10-02](https://doc.rust-lang.org/stable/book/ch10-02-traits.html) |
| Itérateurs (`map`, `filter`, `flat_map`, `collect`, `min_by_key`) | `crates/spacejo-core/src/cards.rs`, `view.rs`, `game.rs` | [ch13-02](https://doc.rust-lang.org/stable/book/ch13-02-iterators.html) |
| Dérivation de traits avec `#[derive]` (serde `Serialize`/`Deserialize`) | `crates/spacejo-core/src/*.rs` | [annexe C](https://doc.rust-lang.org/stable/book/appendix-03-derivable-traits.html) |
| Tests d'intégration (`tests/`, module `common` partagé) | `crates/spacejo-core/tests/` | [ch11-03](https://doc.rust-lang.org/stable/book/ch11-03-test-organization.html) |
| `async`/`await` et `#[tokio::main]` (code asynchrone) | `src/handlers.rs`, `api/*.rs` | [ch17](https://doc.rust-lang.org/stable/book/ch17-00-async-await.html) |
| Fermetures (closures), `FnOnce`, mot-clé `move` | `src/handlers.rs` (`write_as_player`), `src/vercel.rs` | [ch13-01](https://doc.rust-lang.org/stable/book/ch13-01-closures.html) |
| État partagé : `Arc<Mutex<…>>` | `src/store.rs` (`MemoryStore`) | [ch16-03](https://doc.rust-lang.org/stable/book/ch16-03-shared-state.html) |
| Trait `From` et conversion d'erreurs par `?` | `src/error.rs` | [ch09-02](https://doc.rust-lang.org/stable/book/ch09-02-recoverable-errors-with-result.html#a-shortcut-for-propagating-errors-the--operator) |
| Généricité avec contraintes de trait (`T: DeserializeOwned`, `F: FnOnce(...)`) | `src/handlers.rs` | [ch10-01](https://doc.rust-lang.org/stable/book/ch10-01-syntax.html) |
| Tests unitaires `#[cfg(test)]` dans le même fichier | `src/auth.rs` | [ch11-03](https://doc.rust-lang.org/stable/book/ch11-03-test-organization.html) |
| `let … else` (sortir tôt quand un motif ne correspond pas) | `src/handlers.rs`, `crates/spacejo-core/src/game.rs` | [ch19-01](https://doc.rust-lang.org/stable/book/ch19-01-all-the-places-for-patterns.html) |
| Méthodes d'`Option` (`is_none_or`, `is_some_and`, `filter`) | `src/handlers.rs`, `crates/spacejo-core/src/game.rs` | [doc de `Option`](https://doc.rust-lang.org/std/option/enum.Option.html) |

