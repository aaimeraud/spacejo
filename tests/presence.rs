//! Présence des joueurs, attente d'un absent et élimination, vues depuis l'API (stockage en mémoire,
//! horloge simulée). Voir ASSUMPTIONS.md, n° 18 à 21.
mod common;

use std::time::Duration;

use common::{Client, act, create, get_state, join, skip, start, version};
use serde_json::{Value, json};
use spacejo_api::handlers::{ABSENT_TURN_WAIT_SECS, PRESENCE_TIMEOUT_SECS};
use spacejo_api::{MemoryStore, Store};

fn memory() -> (Store, MemoryStore) {
    let memory = MemoryStore::new();
    (Store::Memory(memory.clone()), memory)
}

fn secs(value: f64) -> Duration {
    Duration::from_secs_f64(value)
}

async fn poll(store: &Store, client: &Client) -> Value {
    get_state(store, &format!("code={}", client.code), Some(&client.token))
        .await
        .body
}

/// Partie à 3 joueurs, mise en place terminée.
async fn playing(store: &Store) -> Vec<Client> {
    let (host, _) = create(store, "Alice", None).await;
    let (bob, _) = join(store, &host.code, "Bob").await;
    let (chloe, _) = join(store, &host.code, "Chloé").await;
    let clients = vec![host, bob, chloe];
    assert_eq!(start(store, &clients[0]).await.status, 200);
    for client in &clients {
        for (column, row) in [(0, 0), (0, 1)] {
            let v = version(store, client).await;
            let action = json!({ "type": "RevealInitial", "column": column, "row": row });
            assert_eq!(act(store, client, v, action).await.status, 200);
        }
    }
    clients
}

fn current(body: &Value) -> usize {
    body["view"]["current_player"].as_u64().unwrap() as usize
}

#[tokio::test]
async fn polling_marks_players_present_without_changing_the_version() {
    let (store, memory) = memory();
    let (host, _) = create(&store, "Alice", None).await;
    let (bob, _) = join(&store, &host.code, "Bob").await;
    let before = version(&store, &host).await;
    for _ in 0..3 {
        poll(&store, &bob).await;
    }
    assert_eq!(
        version(&store, &host).await,
        before,
        "la présence n'écrit pas la partie"
    );

    // Bob ne donne plus de nouvelles : il devient absent pour Alice.
    memory.advance(secs(PRESENCE_TIMEOUT_SECS + 1.0));
    let body = poll(&store, &host).await;
    assert_eq!(
        body["presence"],
        json!([{ "player": 0, "connected": true }, { "player": 1, "connected": false }])
    );
    // Il revient.
    poll(&store, &bob).await;
    let body = poll(&store, &host).await;
    assert_eq!(body["presence"][1]["connected"], true);
}

#[tokio::test]
async fn an_absent_player_is_waited_for_then_skipped_then_eliminated() {
    let (store, memory) = memory();
    let clients = playing(&store).await;
    let body = poll(&store, &clients[0]).await;
    let absent = current(&body);
    let present: Vec<&Client> = clients
        .iter()
        .filter(|c| c.player_id as usize != absent)
        .collect();

    // Le joueur dont c'est le tour s'en va. Les autres continuent d'interroger le serveur.
    memory.advance(secs(PRESENCE_TIMEOUT_SECS + 1.0));
    for c in &present {
        poll(&store, c).await;
    }
    let body = poll(&store, present[0]).await;
    assert_eq!(body["waiting"]["player"], absent);
    assert_eq!(body["waiting"]["missed_turns"], 0);
    let left = body["waiting"]["seconds_left"].as_u64().unwrap();
    assert!(left > 0 && left as f64 <= ABSENT_TURN_WAIT_SECS, "{left}");
    assert_eq!(current(&body), absent, "on l'attend encore");

    // Un tour d'attente plus tard, le polling d'un joueur présent passe son tour.
    memory.advance(secs(ABSENT_TURN_WAIT_SECS));
    for c in &present {
        poll(&store, c).await;
    }
    let body = poll(&store, present[0]).await;
    assert_ne!(current(&body), absent, "son tour est passé");
    assert_eq!(body["view"]["players"][absent]["missed_turns"], 1);
    assert_eq!(body["view"]["players"][absent]["eliminated"], false);

    // Les joueurs présents jouent jusqu'à ce que ce soit de nouveau son tour.
    for _ in 0..present.len() {
        let body = poll(&store, present[0]).await;
        let turn = current(&body);
        if turn == absent {
            break;
        }
        let player = clients
            .iter()
            .find(|c| c.player_id as usize == turn)
            .unwrap();
        let v = body["version"].as_i64().unwrap();
        assert_eq!(
            act(&store, player, v, json!({ "type": "DrawFromPile" }))
                .await
                .status,
            200
        );
        let v = version(&store, player).await;
        let r = act(
            &store,
            player,
            v,
            json!({ "type": "SwapHeld", "column": 0, "row": 0 }),
        )
        .await;
        assert_eq!(r.status, 200, "{:?}", r.body);
    }
    let body = poll(&store, present[0]).await;
    assert_eq!(current(&body), absent);

    // Toujours absent : second tour manqué, il est éliminé.
    memory.advance(secs(ABSENT_TURN_WAIT_SECS + 1.0));
    for c in &present {
        poll(&store, c).await;
    }
    let body = poll(&store, present[0]).await;
    assert_eq!(body["view"]["players"][absent]["eliminated"], true);
    assert_ne!(current(&body), absent);

    // S'il revient, il voit la partie mais ne peut plus jouer.
    let gone = clients
        .iter()
        .find(|c| c.player_id as usize == absent)
        .unwrap();
    let body = poll(&store, gone).await;
    assert_eq!(body["view"]["players"][absent]["eliminated"], true);
    let v = body["version"].as_i64().unwrap();
    let refused = act(&store, gone, v, json!({ "type": "DrawFromPile" })).await;
    assert_eq!(refused.status, 422);
    assert_eq!(refused.body["error"]["code"], "eliminated");
}

#[tokio::test]
async fn a_present_player_is_never_skipped() {
    let (store, memory) = memory();
    let clients = playing(&store).await;
    let body = poll(&store, &clients[0]).await;
    let turn = current(&body);
    let v = body["version"].as_i64().unwrap();
    // Tout le monde reste là (polling toutes les 2 s, comme l'interface), mais le joueur courant
    // réfléchit plus longtemps qu'un « tour » d'attente.
    let polls = (2.0 * ABSENT_TURN_WAIT_SECS / 2.0) as usize;
    for _ in 0..polls {
        memory.advance(secs(2.0));
        for c in &clients {
            poll(&store, c).await;
        }
    }
    let body = poll(&store, &clients[0]).await;
    assert_eq!(current(&body), turn);
    assert_eq!(body["version"].as_i64().unwrap(), v);
    assert_eq!(body["waiting"], json!(null));
}

/// Joue le tour du joueur courant (pioche, puis échange avec sa première carte).
async fn play_turn(store: &Store, clients: &[Client]) {
    let body = poll(store, &clients[0]).await;
    let player = &clients[current(&body)];
    let v = version(store, player).await;
    assert_eq!(
        act(store, player, v, json!({ "type": "DrawFromPile" }))
            .await
            .status,
        200
    );
    let v = version(store, player).await;
    let swap = json!({ "type": "SwapHeld", "column": 0, "row": 0 });
    assert_eq!(act(store, player, v, swap).await.status, 200);
}

/// Partie à 3 joueurs où le joueur courant n'est pas l'hôte et vient de partir.
async fn absent_opponent(store: &Store, memory: &MemoryStore) -> (Vec<Client>, usize) {
    let clients = playing(store).await;
    if current(&poll(store, &clients[0]).await) == 0 {
        play_turn(store, &clients).await;
    }
    let absent = current(&poll(store, &clients[0]).await);
    memory.advance(secs(PRESENCE_TIMEOUT_SECS + 1.0));
    for c in clients.iter().filter(|c| c.player_id as usize != absent) {
        poll(store, c).await;
    }
    (clients, absent)
}

#[tokio::test]
async fn the_host_can_skip_an_absent_player_without_waiting() {
    let (store, memory) = memory();
    let (clients, absent) = absent_opponent(&store, &memory).await;
    let body = poll(&store, &clients[0]).await;
    assert_eq!(body["waiting"]["player"], absent);
    let v = body["version"].as_i64().unwrap();

    let skipped = skip(&store, &clients[0], v, absent).await;
    assert_eq!(skipped.status, 200, "{:?}", skipped.body);
    assert_eq!(skipped.body["version"].as_i64().unwrap(), v + 1);
    assert_ne!(
        current(&skipped.body),
        absent,
        "son tour est passé tout de suite"
    );
    assert_eq!(skipped.body["view"]["players"][absent]["missed_turns"], 1);
}

#[tokio::test]
async fn only_the_host_skips_and_only_the_absent_player_who_blocks() {
    let (store, memory) = memory();
    let (clients, absent) = absent_opponent(&store, &memory).await;
    let v = version(&store, &clients[0]).await;
    let other = (1..clients.len()).find(|&p| p != absent).unwrap();

    // Un joueur qui n'est pas l'hôte : refusé.
    let refused = skip(&store, &clients[other], v, absent).await;
    assert_eq!(refused.status, 403);
    assert_eq!(refused.body["error"]["code"], "not_host");

    // Un joueur présent, ou qui ne bloque pas la partie : refusé.
    for target in [0, other] {
        let refused = skip(&store, &clients[0], v, target).await;
        assert_eq!(refused.status, 422, "{target}");
        assert_eq!(refused.body["error"]["code"], "player_not_absent");
    }
    // Joueur inconnu : refusé de la même façon.
    let refused = skip(&store, &clients[0], v, 9).await;
    assert_eq!(refused.body["error"]["code"], "player_not_absent");

    // Écran périmé : refusé, rien n'est écrit.
    let refused = skip(&store, &clients[0], v - 1, absent).await;
    assert_eq!(refused.status, 409);
    assert_eq!(version(&store, &clients[0]).await, v);
}
