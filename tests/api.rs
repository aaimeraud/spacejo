//! Tests de l'API avec le stockage en mémoire (critères 9, 10, 11, 12, 16, 17).
mod common;

use common::{Client, act, assert_no_hidden_values, call, create, get_state, join, start, version};
use serde_json::{Value, json};
use spacejo_api::auth::{generate_token, hash_token};
use spacejo_api::{Endpoint, MemoryStore, Store, StoredGame};
use spacejo_core::{Action, Game, Rules};

fn memory() -> (Store, MemoryStore) {
    let memory = MemoryStore::new();
    (Store::Memory(memory.clone()), memory)
}

/// Partie lancée à 2 joueurs, mise en place terminée. Renvoie (hôte, invité, version).
async fn playing_game(store: &Store) -> (Client, Client, i64) {
    let (host, _) = create(store, "Alice", None).await;
    let (guest, _) = join(store, &host.code, "Bob").await;
    assert_eq!(start(store, &host).await.status, 200);
    for client in [&host, &guest] {
        for (column, row) in [(0, 0), (0, 1)] {
            let v = version(store, client).await;
            let response = act(
                store,
                client,
                v,
                json!({ "type": "RevealInitial", "column": column, "row": row }),
            )
            .await;
            assert_eq!(response.status, 200, "{:?}", response.body);
        }
    }
    let v = version(store, &host).await;
    (host, guest, v)
}

fn current_player(body: &Value) -> u64 {
    body["view"]["current_player"].as_u64().unwrap()
}

// ----- Création, jonction, état -----

#[tokio::test]
async fn create_join_and_read_the_state() {
    let (store, _) = memory();
    let (host, response) = create(&store, "Alice", None).await;
    assert_eq!(host.code.len(), 6);
    assert_eq!(host.token.len(), 64);
    assert_eq!(host.player_id, 0);
    assert_eq!(response.body["version"], 1);
    assert_eq!(response.body["view"]["phase"], "Lobby");

    // Le code est accepté en minuscules.
    let (guest, response) = join(&store, &host.code.to_lowercase(), "Bob").await;
    assert_eq!(guest.player_id, 1);
    assert_eq!(response.body["version"], 2);
    assert_eq!(response.body["view"]["me"], 1);
    assert_eq!(response.body["view"]["players"][1]["name"], "Bob");

    // Lecture complète, puis lecture minimale quand rien n'a changé.
    let query = format!("code={}", host.code);
    let full = get_state(&store, &query, Some(&host.token)).await;
    assert_eq!(full.status, 200);
    assert_eq!(full.body["changed"], true);
    assert_eq!(full.body["version"], 2);
    let minimal = get_state(
        &store,
        &format!("{query}&known_version=2"),
        Some(&host.token),
    )
    .await;
    // Réponse minimale : pas de vue, seulement la version et la présence des joueurs.
    assert_eq!(minimal.body["changed"], false);
    assert_eq!(minimal.body["version"], 2);
    assert!(minimal.body.get("view").is_none());
    assert_eq!(
        minimal.body["presence"],
        json!([{ "player": 0, "connected": true }, { "player": 1, "connected": true }])
    );
    assert_eq!(minimal.body["waiting"], json!(null));
    let stale = get_state(
        &store,
        &format!("{query}&known_version=1"),
        Some(&host.token),
    )
    .await;
    assert_eq!(stale.body["changed"], true);
}

#[tokio::test]
async fn bad_requests_are_refused_with_clear_errors() {
    let (store, _) = memory();
    let bad_json = spacejo_api::handle(
        &store,
        Endpoint::Create,
        spacejo_api::ApiRequest {
            method: "POST".into(),
            body: b"{pas du json".to_vec(),
            ..Default::default()
        },
    )
    .await;
    assert_eq!(bad_json.status, 400);
    assert_eq!(bad_json.body["error"]["code"], "invalid_request");

    let unknown_field = call(
        &store,
        Endpoint::Create,
        json!({ "name": "A", "admin": true }),
        None,
    )
    .await;
    assert_eq!(unknown_field.status, 400);

    let wrong_method = spacejo_api::handle(
        &store,
        Endpoint::Create,
        spacejo_api::ApiRequest {
            method: "GET".into(),
            ..Default::default()
        },
    )
    .await;
    assert_eq!(wrong_method.status, 405);

    let empty_name = call(&store, Endpoint::Create, json!({ "name": "  " }), None).await;
    assert_eq!(empty_name.status, 400);
    assert_eq!(empty_name.body["error"]["code"], "invalid_name");

    let bad_code = call(
        &store,
        Endpoint::Join,
        json!({ "code": "OOOOOO", "name": "B" }),
        None,
    )
    .await;
    assert_eq!(bad_code.body["error"]["code"], "invalid_code");
    let missing = call(
        &store,
        Endpoint::Join,
        json!({ "code": "ABCDEF", "name": "B" }),
        None,
    )
    .await;
    assert_eq!(missing.status, 404);
    assert_eq!(missing.body["error"]["code"], "game_not_found");

    let (host, _) = create(&store, "Alice", None).await;
    let no_code = get_state(&store, "", Some(&host.token)).await;
    assert_eq!(no_code.status, 400);
    let bad_version = get_state(
        &store,
        &format!("code={}&known_version=x", host.code),
        Some(&host.token),
    )
    .await;
    assert_eq!(bad_version.status, 400);
    let unknown_param = get_state(
        &store,
        &format!("code={}&debug=1", host.code),
        Some(&host.token),
    )
    .await;
    assert_eq!(unknown_param.status, 400);
}

#[tokio::test]
async fn joining_is_limited_to_ten_players_and_to_the_lobby() {
    let (store, _) = memory();
    let (host, _) = create(&store, "P0", None).await;
    for i in 1..spacejo_core::MAX_PLAYERS {
        join(&store, &host.code, &format!("P{i}")).await;
    }
    let full = call(
        &store,
        Endpoint::Join,
        json!({ "code": host.code, "name": "En trop" }),
        None,
    )
    .await;
    assert_eq!(full.status, 422);
    assert_eq!(full.body["error"]["code"], "game_full");

    let (store, _) = memory();
    let (host, _) = create(&store, "P0", None).await;
    join(&store, &host.code, "P1").await;
    start(&store, &host).await;
    let late = call(
        &store,
        Endpoint::Join,
        json!({ "code": host.code, "name": "P2" }),
        None,
    )
    .await;
    assert_eq!(late.body["error"]["code"], "wrong_phase");
}

// ----- Critère 16 : réglages -----

#[tokio::test]
async fn creating_without_rules_uses_the_official_rules_and_stores_them_all() {
    let (store, memory) = memory();
    let (host, response) = create(&store, "Alice", None).await;
    let defaults = serde_json::to_value(Rules::default()).unwrap();
    assert_eq!(response.body["view"]["rules"], defaults);
    // L'état enregistré contient les 8 réglages, pas seulement ceux qui ont été choisis.
    let raw = memory.raw_state(&host.code).unwrap();
    assert_eq!(raw["game"]["rules"], defaults);
    assert_eq!(raw["game"]["rules"].as_object().unwrap().len(), 8);
    assert_eq!(raw["state_version"], spacejo_api::STATE_VERSION);
}

#[tokio::test]
async fn creating_with_rules_validates_every_setting() {
    let (store, memory) = memory();
    let variant = json!({
        "penalty": "DoubleOnlyIfBeaten",
        "penalty_scope": "PositiveCardsOnly",
        "first_player_tie": "Random",
        "next_round_starter": "Rotate",
        "empty_pile": "EndRound",
        "end_tie": "ExtraRound",
        "end_score": 150,
        "deck": "Reduced"
    });
    let (host, response) = create(&store, "Alice", Some(variant.clone())).await;
    assert_eq!(response.body["view"]["rules"], variant);
    assert_eq!(
        memory.raw_state(&host.code).unwrap()["game"]["rules"],
        variant
    );

    for field in [
        "penalty",
        "penalty_scope",
        "first_player_tie",
        "next_round_starter",
        "empty_pile",
        "end_tie",
        "deck",
    ] {
        let mut rules = variant.clone();
        rules[field] = json!("Inconnu");
        let response = call(
            &store,
            Endpoint::Create,
            json!({ "name": "A", "rules": rules }),
            None,
        )
        .await;
        assert_eq!(
            response.status, 400,
            "valeur inconnue acceptée pour {field}"
        );
        assert_eq!(response.body["error"]["code"], "invalid_request");
    }
    for end_score in [9, 501, -5] {
        let mut rules = variant.clone();
        rules["end_score"] = json!(end_score);
        let response = call(
            &store,
            Endpoint::Create,
            json!({ "name": "A", "rules": rules }),
            None,
        )
        .await;
        assert_eq!(response.status, 400);
        assert_eq!(response.body["error"]["code"], "invalid_rules");
    }
    // Réglages incomplets : refusés (ils doivent être complets s'ils sont fournis).
    let response = call(
        &store,
        Endpoint::Create,
        json!({ "name": "A", "rules": { "end_score": 50 } }),
        None,
    )
    .await;
    assert_eq!(response.status, 400);
}

// ----- Critère 9 : réglages figés, hôte seulement -----

#[tokio::test]
async fn only_the_host_changes_rules_and_only_before_the_start() {
    let (store, _) = memory();
    let (host, _) = create(&store, "Alice", None).await;
    let (guest, _) = join(&store, &host.code, "Bob").await;
    let mut variant = serde_json::to_value(Rules::default()).unwrap();
    variant["penalty"] = json!("Disabled");

    let v = version(&store, &host).await;
    let by_guest = call(
        &store,
        Endpoint::Rules,
        json!({ "code": host.code, "expected_version": v, "rules": variant }),
        Some(&guest.token),
    )
    .await;
    assert_eq!(by_guest.status, 403);
    assert_eq!(by_guest.body["error"]["code"], "not_host");

    let mut unknown = variant.clone();
    unknown["empty_pile"] = json!("Ignore");
    let unknown_value = call(
        &store,
        Endpoint::Rules,
        json!({ "code": host.code, "expected_version": v, "rules": unknown }),
        Some(&host.token),
    )
    .await;
    assert_eq!(unknown_value.status, 400);

    let by_host = call(
        &store,
        Endpoint::Rules,
        json!({ "code": host.code, "expected_version": v, "rules": variant }),
        Some(&host.token),
    )
    .await;
    assert_eq!(by_host.status, 200);
    assert_eq!(by_host.body["view"]["rules"]["penalty"], "Disabled");
    assert_eq!(by_host.body["version"], v + 1);

    // Le non-hôte ne peut pas non plus lancer la partie.
    let v = version(&store, &host).await;
    let start_by_guest = call(
        &store,
        Endpoint::Start,
        json!({ "code": host.code, "expected_version": v }),
        Some(&guest.token),
    )
    .await;
    assert_eq!(start_by_guest.body["error"]["code"], "not_host");

    assert_eq!(start(&store, &host).await.status, 200);
    let v = version(&store, &host).await;
    let after_start = call(
        &store,
        Endpoint::Rules,
        json!({ "code": host.code, "expected_version": v, "rules": serde_json::to_value(Rules::default()).unwrap() }),
        Some(&host.token),
    )
    .await;
    assert_eq!(after_start.status, 422);
    assert_eq!(after_start.body["error"]["code"], "wrong_phase");
}

// ----- Critère 11 : jetons -----

#[tokio::test]
async fn requests_without_a_valid_token_are_refused() {
    let (store, _) = memory();
    let (host, guest, v) = playing_game(&store).await;
    let draw = json!({ "type": "DrawFromPile" });

    let no_token = call(
        &store,
        Endpoint::Action,
        json!({ "code": host.code, "expected_version": v, "action": draw }),
        None,
    )
    .await;
    assert_eq!(no_token.status, 401);
    assert_eq!(no_token.body["error"]["code"], "missing_token");

    let empty = call(
        &store,
        Endpoint::Action,
        json!({ "code": host.code, "expected_version": v, "action": draw }),
        Some(""),
    )
    .await;
    assert_eq!(empty.status, 401);

    let forged = Client {
        token: generate_token().unwrap(),
        ..host.clone()
    };
    let wrong = act(&store, &forged, v, draw.clone()).await;
    assert_eq!(wrong.status, 401);
    assert_eq!(wrong.body["error"]["code"], "invalid_token");

    // Le hachage d'un jeton n'est pas un jeton.
    let hash_as_token = Client {
        token: hash_token(&host.token),
        ..host.clone()
    };
    assert_eq!(
        act(&store, &hash_as_token, v, draw.clone()).await.status,
        401
    );

    // Un jeton d'une autre partie ne vaut rien ici.
    let (other_host, _) = create(&store, "Zoé", None).await;
    let foreign = Client {
        token: other_host.token,
        ..host.clone()
    };
    assert_eq!(act(&store, &foreign, v, draw.clone()).await.status, 401);
    assert_eq!(
        get_state(&store, &format!("code={}", host.code), Some(&foreign.token))
            .await
            .status,
        401
    );
    assert_eq!(
        get_state(&store, &format!("code={}", host.code), None)
            .await
            .status,
        401
    );

    // Le jeton d'un joueur ne permet pas de jouer à la place de l'autre : avec son propre jeton,
    // le joueur dont ce n'est pas le tour est refusé.
    let state = get_state(&store, &format!("code={}", host.code), Some(&host.token)).await;
    let current = current_player(&state.body);
    let not_current = if current == host.player_id {
        &guest
    } else {
        &host
    };
    let refused = act(&store, not_current, v, draw.clone()).await;
    assert_eq!(refused.status, 422);
    assert_eq!(refused.body["error"]["code"], "not_your_turn");
    assert_eq!(version(&store, &host).await, v, "rien n'a été écrit");
}

// ----- Critères 12 et 17 : versions et conflits -----

#[tokio::test]
async fn a_stale_expected_version_is_refused_and_writes_nothing() {
    let (store, memory) = memory();
    let (host, guest, v) = playing_game(&store).await;
    let state = get_state(&store, &format!("code={}", host.code), Some(&host.token)).await;
    let current = if current_player(&state.body) == 0 {
        &host
    } else {
        &guest
    };
    let before = memory.raw_state(&host.code).unwrap();

    let stale = act(&store, current, v - 1, json!({ "type": "DrawFromPile" })).await;
    assert_eq!(stale.status, 409);
    assert_eq!(stale.body["error"]["code"], "version_conflict");
    assert_eq!(
        memory.raw_state(&host.code).unwrap(),
        before,
        "rien n'est écrit"
    );
    assert_eq!(version(&store, &host).await, v);

    // Réglages avec une version périmée : même refus (testé au salon d'une autre partie).
    let (lobby_host, _) = create(&store, "Hôte", None).await;
    let rules = serde_json::to_value(Rules::default()).unwrap();
    let stale_rules = call(
        &store,
        Endpoint::Rules,
        json!({ "code": lobby_host.code, "expected_version": 0, "rules": rules }),
        Some(&lobby_host.token),
    )
    .await;
    assert_eq!(stale_rules.status, 409);
}

#[tokio::test]
async fn a_replayed_request_is_applied_only_once() {
    let (store, _) = memory();
    let (host, guest, v) = playing_game(&store).await;
    let state = get_state(&store, &format!("code={}", host.code), Some(&host.token)).await;
    let current = if current_player(&state.body) == 0 {
        &host
    } else {
        &guest
    };
    let draw = json!({ "type": "DrawFromPile" });

    let first = act(&store, current, v, draw.clone()).await;
    assert_eq!(first.status, 200);
    let replay = act(&store, current, v, draw).await;
    assert_eq!(replay.status, 409, "la même requête rejouée est refusée");
    assert_eq!(version(&store, &host).await, v + 1);
}

#[tokio::test]
async fn two_actions_on_the_same_version_only_one_succeeds() {
    let (store, _) = memory();
    let (host, guest, v) = playing_game(&store).await;
    let state = get_state(&store, &format!("code={}", host.code), Some(&host.token)).await;
    let current = if current_player(&state.body) == 0 {
        &host
    } else {
        &guest
    };
    // Deux onglets du même joueur envoient chacun une action différente sur la même version.
    let (a, b) = tokio::join!(
        act(&store, current, v, json!({ "type": "DrawFromPile" })),
        act(&store, current, v, json!({ "type": "TakeDiscard" })),
    );
    let statuses = [a.status, b.status];
    assert!(
        statuses.contains(&200) && statuses.contains(&409),
        "{statuses:?}"
    );
}

// ----- Critère 10 : aucune information cachée dans les réponses -----

/// Partie lancée (mise en place terminée) avec deux jetons connus.
fn known_game(tokens: &[String]) -> StoredGame {
    let mut game = Game::new("Alice", Rules::default(), 99).unwrap();
    game.join("Bob").unwrap();
    game.start(0).unwrap();
    for p in 0..2 {
        game.apply(p, Action::RevealInitial { column: 0, row: 0 })
            .unwrap();
        game.apply(p, Action::RevealInitial { column: 0, row: 1 })
            .unwrap();
    }
    StoredGame::new(game, tokens.iter().map(|t| hash_token(t)).collect())
}

#[tokio::test]
async fn responses_do_not_change_when_only_a_hidden_card_changes() {
    let tokens = vec![generate_token().unwrap(), generate_token().unwrap()];
    let original = known_game(&tokens);
    let mut altered = original.clone();
    let (c, r) = altered.game.players[1].grid.first_hidden().unwrap();
    let cell = altered.game.players[1].grid.cell_mut(c, r).unwrap();
    cell.value = if cell.value == 12 { -2 } else { 12 };

    let mut bodies = Vec::new();
    for stored in [&original, &altered] {
        let memory = MemoryStore::new();
        memory.put_raw("K7MPQX", stored.encode().unwrap()).unwrap();
        let store = Store::Memory(memory);
        let response = get_state(&store, "code=K7MPQX", Some(&tokens[0])).await;
        assert_eq!(response.status, 200);
        bodies.push(serde_json::to_string(&response.body).unwrap());
    }
    assert_eq!(
        bodies[0], bodies[1],
        "la vue du joueur 0 ne dépend pas de la carte cachée du joueur 1"
    );
}

#[tokio::test]
async fn no_response_contains_a_hidden_value_or_a_token_hash() {
    let (store, memory) = memory();
    let mut responses = Vec::new();
    let (host, r) = create(&store, "Alice", None).await;
    responses.push(r);
    let (guest, r) = join(&store, &host.code, "Bob").await;
    responses.push(r);
    responses.push(start(&store, &host).await);
    for client in [&host, &guest] {
        let v = version(&store, client).await;
        responses.push(
            act(
                &store,
                client,
                v,
                json!({ "type": "RevealInitial", "column": 1, "row": 1 }),
            )
            .await,
        );
        let v = version(&store, client).await;
        responses.push(
            act(
                &store,
                client,
                v,
                json!({ "type": "RevealInitial", "column": 2, "row": 2 }),
            )
            .await,
        );
    }
    // Quelques coups.
    for _ in 0..6 {
        let state = get_state(&store, &format!("code={}", host.code), Some(&host.token)).await;
        let current = if current_player(&state.body) == 0 {
            &host
        } else {
            &guest
        };
        let v = state.body["version"].as_i64().unwrap();
        responses.push(act(&store, current, v, json!({ "type": "DrawFromPile" })).await);
        let state = get_state(&store, &format!("code={}", host.code), Some(&current.token)).await;
        responses.push(state.clone());
        let grid = &state.body["view"]["players"][current.player_id as usize]["grid"];
        let (column, row) = first_hidden_in_view(grid);
        let v = state.body["version"].as_i64().unwrap();
        responses.push(
            act(
                &store,
                current,
                v,
                json!({ "type": "DiscardHeldAndReveal", "column": column, "row": row }),
            )
            .await,
        );
    }
    for client in [&host, &guest] {
        responses
            .push(get_state(&store, &format!("code={}", host.code), Some(&client.token)).await);
    }

    let raw = memory.raw_state(&host.code).unwrap();
    let hashes = common::stored_hashes(&raw);
    assert_eq!(hashes.len(), 2);
    // L'état stocké contient les hachages mais jamais les jetons eux-mêmes.
    let raw_text = raw.to_string();
    assert!(!raw_text.contains(&host.token) && !raw_text.contains(&guest.token));

    for response in &responses {
        assert!(response.status < 300, "{:?}", response.body);
        assert_no_hidden_values(&response.body);
        let text = response.body.to_string();
        for hash in &hashes {
            assert!(
                !text.contains(hash.as_str()),
                "hachage de jeton dans une réponse"
            );
        }
        assert!(
            !text.contains("token_hashes")
                && !text.contains("rng")
                && !text.contains("draw_pile\"")
        );
    }
}

fn first_hidden_in_view(grid: &Value) -> (usize, usize) {
    for (c, column) in grid.as_array().unwrap().iter().enumerate() {
        if let Some(slots) = column["slots"].as_array() {
            for (r, slot) in slots.iter().enumerate() {
                if slot["kind"] == "Hidden" {
                    return (c, r);
                }
            }
        }
    }
    panic!("aucune carte cachée");
}
