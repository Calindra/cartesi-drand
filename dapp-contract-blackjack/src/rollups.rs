pub mod rollup {
    use hyper::{body::to_bytes, header, Body, Client, Method, Request, StatusCode};
    use log::{error, info, warn};
    use serde_json::{from_str, json, Value};
    use std::{env, error::Error, str::from_utf8, sync::Arc, time::Duration};
    use tokio::sync::Mutex;

    use crate::{
        models::{
            game::prelude::{Manager, Table},
            player::{check_fields_create_player, prelude::Player},
        },
        util::{
            json::{
                decode_payload, generate_report, get_address_metadata_from_root, get_path_player,
                get_path_player_name, load_json, write_json,
            },
            pubkey::{call_update_key, DrandEnv},
            random::retrieve_seed,
        },
    };

    pub async fn rollup(manager: Arc<Mutex<Manager>>) -> Result<(), Box<dyn Error>> {
        info!("Starting loop...");

        let client = Client::new();
        let server_addr = env::var("MIDDLEWARE_HTTP_SERVER_URL")?;

        let mut status = "accept";
        loop {
            info!("Sending finish");
            let response = json!({ "status": status });
            let request = Request::builder()
                .method(Method::POST)
                .header(header::CONTENT_TYPE, "application/json")
                .uri(format!("{}/finish", &server_addr))
                .body(Body::from(response.to_string()))?;
            let response = client.request(request).await?;
            let status_response = response.status();
            info!("Receive finish status {}", &status_response);

            if status_response == StatusCode::ACCEPTED {
                warn!("No pending rollup request, trying again");
            } else {
                let body = to_bytes(response).await?;
                let body = from_utf8(&body)?;
                let body = from_str::<Value>(body)?;

                let request_type = body["request_type"]
                    .as_str()
                    .ok_or("request_type is not a string")?;

                status = match request_type {
                    "advance_state" => {
                        handle_advance(manager.clone(), &server_addr[..], body).await?
                    }
                    "inspect_state" => {
                        handle_inspect(manager.clone(), &server_addr[..], body).await?
                    }
                    &_ => {
                        error!("Unknown request type");
                        "reject"
                    }
                }
            }
            #[cfg(not(target_arch = "riscv64"))]
            wait_func().await;
        }
    }

    pub async fn wait_func() {
        #[cfg(not(target_arch = "riscv64"))]
        {
            warn!("waiting 5s...");
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    async fn handle_body(manager: Arc<Mutex<Manager>>, body: &Value) -> Result<(), Box<dyn Error>> {
        info!("body {:}", &body);

        let result = handle_request_action(body, manager, true).await?;

        if let Some(report) = result {
            send_report(report).await?;
        }

        Ok(())
    }

    pub async fn handle_inspect(
        manager: Arc<Mutex<Manager>>,
        _server_addr: &str,
        body: Value,
    ) -> Result<&'static str, Box<dyn Error>> {
        info!("Handling inspect");

        handle_body(manager, &body).await?;

        Ok("accept")
    }

    pub async fn handle_advance(
        manager: Arc<Mutex<Manager>>,
        _server_addr: &str,
        body: Value,
    ) -> Result<&'static str, Box<dyn Error>> {
        info!("Handling advance");

        handle_body(manager, &body).await?;

        Ok("accept")
    }

    pub async fn send_report(report: Value) -> Result<&'static str, Box<dyn std::error::Error>> {
        let server_addr = std::env::var("ROLLUP_HTTP_SERVER_URL")?;
        let client = hyper::Client::new();
        let req = hyper::Request::builder()
            .method(hyper::Method::POST)
            .header(hyper::header::CONTENT_TYPE, "application/json")
            .uri(format!("{}/report", server_addr))
            .body(hyper::Body::from(report.to_string()))?;

        let _ = client.request(req).await?;
        Ok("accept")
    }

    pub async fn send_notice(notice: Value) -> Result<(), Box<dyn std::error::Error>> {
        let server_addr = std::env::var("ROLLUP_HTTP_SERVER_URL")?;
        let client = hyper::Client::new();
        let req = hyper::Request::builder()
            .method(hyper::Method::POST)
            .header(hyper::header::CONTENT_TYPE, "application/json")
            .uri(format!("{}/notice", server_addr))
            .body(hyper::Body::from(notice.to_string()))?;

        let result = client.request(req).await?;
        info!("Send notice: {:?}", result.body());
        Ok(())
    }

    pub fn get_payload_from_root(root: &Value) -> Option<Value> {
        let root = root.as_object()?;
        let root = root.get("data")?.as_object()?;
        let payload = root.get("payload")?.as_str()?;
        let payload = decode_payload(payload)?;
        Some(payload)
    }

    pub fn get_from_payload_action(payload: &Value) -> Option<String> {
        let input = payload.get("input")?.as_object()?;
        let action = input.get("action")?.as_str()?;
        Some(action.to_owned())
    }

    async fn async_pick(table: Arc<Mutex<Table>>, player_id: String, timestamp: u64) {
        info!("Player calling: {}", player_id);
        // start game stop here
        let seed = match retrieve_seed(timestamp).await {
            Ok(seed) => seed,
            Err(_) => {
                // here in prod mode we got an infinite loop, so we need a break when an inspect arrives.
                // after the inspect ends the cartesi machine do a time travel to the retrieve_seed point.
                return;
            }
        };

        let result = table
            .lock()
            .await
            .hit_player(&player_id, timestamp, &seed)
            .await;

        if let Err(err) = result {
            error!("Pick error: {:}", err);
        }
    }

    pub async fn handle_request_action(
        root: &Value,
        manager: Arc<Mutex<Manager>>,
        write_hd_mode: bool,
    ) -> Result<Option<Value>, &'static str> {
        let payload = get_payload_from_root(root).ok_or("Invalid payload")?;
        let action = get_from_payload_action(&payload);

        info!("Action: {:}", action.as_deref().unwrap_or("None"));

        match action.as_deref() {
            Some("update_drand") => {
                let input = payload.get("input").ok_or("Invalid field input")?;

                let metadata = get_address_metadata_from_root(root).ok_or("Invalid address")?;
                let address_owner = metadata.address.trim_start_matches("0x").to_lowercase();

                let address_owner_game =
                    env::var("ADDRESS_OWNER_GAME").or(Err("Address owner game not defined"))?;

                let address_owner_game = address_owner_game.trim_start_matches("0x").to_lowercase();

                if address_owner != address_owner_game {
                    return Err("Invalid owner");
                }

                // Parsing JSON
                let public_key = input
                    .get("public_key")
                    .ok_or("Invalid field public_key")?
                    .as_str()
                    .ok_or("Invalid public_key")?;

                let period = input.get("period").map(|v| v.as_u64()).unwrap_or(None);

                let genesis_time = input
                    .get("genesis_time")
                    .map(|v| v.as_u64())
                    .unwrap_or(None);

                let safe_seconds = input
                    .get("safe_seconds")
                    .map(|v| v.as_u64())
                    .unwrap_or(None);

                let request_env = DrandEnv::new(public_key, period, genesis_time, safe_seconds);

                let response = call_update_key(&request_env).await;

                if response.is_err() {
                    return Err("Could not update drand config");
                }
            }
            Some("new_player") => {
                let input = payload.get("input").ok_or("Invalid field input")?;
                let player_name = check_fields_create_player(input)?;

                let encoded_name = bs58::encode(&player_name).into_string();

                let metadata = get_address_metadata_from_root(root).ok_or("Invalid address")?;
                let address_owner = metadata.address.trim_start_matches("0x");
                let address_encoded = bs58::encode(address_owner).into_string();

                // Add player to manager
                let player = Player::new(address_encoded.clone(), player_name.to_string());
                let mut manager = manager.lock().await;
                let player = Arc::new(player);
                manager.add_player(player)?;

                // Persist player
                if write_hd_mode {
                    let address_owner_obj =
                        json!({ "address": address_owner, "name": player_name });
                    let address_path = get_path_player(&address_encoded);

                    write_json(&address_path, &address_owner_obj)
                        .await
                        .or(Err("Could not write address"))?;

                    let player_path = get_path_player_name(&encoded_name);
                    let player = json!({ "name": encoded_name, "address": metadata.address });
                    write_json(&player_path, &player)
                        .await
                        .or(Err("Could not write player"))?;
                }

                let report = generate_report(json!({
                    "address": address_encoded,
                    "encoded_name": encoded_name,
                    "name": player_name,
                }));

                info!("Report: {:}", report);

                return Ok(Some(report));
            }
            Some("join_game") => {
                let input = payload.get("input").ok_or("Invalid field input")?;

                // Address
                let metadata = get_address_metadata_from_root(root).ok_or("Invalid address")?;
                let address_owner = metadata.address.trim_start_matches("0x");
                let address_encoded = bs58::encode(address_owner).into_string();

                // load to memory if not exists
                if write_hd_mode {
                    load_player_to_mem(&manager, &address_encoded).await?;
                }

                let mut manager = manager.lock().await;
                let player = manager.get_player_ref(&address_encoded)?;

                // Parsing JSON
                let game_id = input
                    .get("game_id")
                    .ok_or("Invalid field game_id")?
                    .as_str()
                    .ok_or("Invalid game_id")?;

                manager.player_join(game_id, player.clone())?;
                info!("Player joined: name {} game_id {}", player.name, game_id);
            }
            Some("show_player") => {
                let input = payload.get("input").ok_or("Invalid field input")?;

                // Parsing JSON
                let address = input
                    .get("address")
                    .ok_or("Invalid field address")?
                    .as_str()
                    .ok_or("Invalid address")?;
                let address_owner = address.trim_start_matches("0x");
                let address_encoded = bs58::encode(address_owner).into_string();

                // load to memory if not exists
                if write_hd_mode {
                    load_player_to_mem(&manager, &address_encoded).await?;
                }

                let manager = manager.lock().await;

                let playing = manager
                    .tables
                    .values()
                    .filter(|&table| table.has_player(&address_encoded))
                    .map(|table| table.get_id())
                    .collect::<Vec<_>>();

                let joined = manager
                    .games
                    .iter()
                    .filter(|&game| game.has_player(&address_encoded))
                    .map(|game| game.get_id())
                    .collect::<Vec<_>>();

                let player_borrow = manager.get_player_by_id(&address_encoded)?;

                let player = json!({
                    "name": player_borrow.name.clone(),
                    "address": address_owner,
                    "joined": joined,
                    "playing": playing,
                });
                info!("player {:?}", player);
                let report = generate_report(player);

                return Ok(Some(report));
            }
            Some("show_games") => {
                let manager = manager.lock().await;
                let report = Manager::generate_games_report(&manager.games);

                info!("ShowGameReport: {:}", report);

                return Ok(Some(report));
            }
            Some("start_game") => {
                let input = payload.get("input").ok_or("Invalid field input")?;
                let metadata = get_address_metadata_from_root(root).ok_or("Invalid address")?;
                let timestamp = metadata.timestamp;

                // Parsing JSON
                let game_id = input
                    .get("game_id")
                    .ok_or("Invalid field game_id")?
                    .as_str()
                    .ok_or("Invalid game_id")?;

                // let deck_nth = input
                //     .get("deck_nth")
                //     .map(|v| v.as_u64().ok_or("Invalid deck_nth"))
                //     .unwrap_or(Ok(2))?;

                let mut manager = manager.lock().await;

                // Get game and make owner
                let game = manager.drop_game(game_id)?;

                // TODO Change here
                if game.players.len() < 2 {
                    manager.add_game(game);
                    return Err("Minimum number of players not reached.");
                }

                let players = game.players.iter().map(|p| p.get_id()).collect::<Vec<_>>();

                // Generate table from game
                let table = game.round_start(2, metadata.timestamp)?;
                let table = Arc::new(Mutex::from(table));

                // Draw two cards for each player
                for _ in 0..2 {
                    for player_id in players.iter() {
                        let table = table.clone();
                        let player_id = player_id.to_owned();
                        async_pick(table.clone(), player_id, timestamp).await;
                    }
                }

                let table = Arc::into_inner(table).ok_or("Could not get table")?;
                let table = Mutex::into_inner(table);
                let table_id = table.get_id().to_owned();

                // Add table to manager
                manager.add_table(table);
                info!("Game started: game_id {} table_id {}", game_id, table_id);
            }
            Some("stop_game") => {
                let input = payload.get("input").ok_or("Invalid field input")?;

                // Parsing JSON
                let game_id = input
                    .get("game_id")
                    .ok_or("Invalid field game_id")?
                    .as_str()
                    .ok_or("Invalid game_id")?;

                let mut manager = manager.lock().await;

                manager.stop_game(game_id).await?;
            }
            Some("show_hands") => {
                let input = payload.get("input").ok_or("Invalid field input")?;

                // Parsing JSON
                let table_id = input
                    .get("table_id")
                    .ok_or("Invalid field table_id")?
                    .as_str()
                    .ok_or("Invalid table_id")?;

                let manager = manager.lock().await;

                if let Some(table) = manager.get_table(table_id) {
                    let hands = table.generate_hands();
                    let report = generate_report(hands);

                    // cache
                    // let report = table.get_report_hand();

                    info!("Report enviado do show_hands");

                    return Ok(Some(report));
                }

                info!("Finding score by table_id {} ...", table_id);
                if let Ok(scoreboard) = manager.get_scoreboard(table_id) {
                    let report = generate_report(scoreboard.to_json());

                    info!("Report: {:}", report);

                    return Ok(Some(report));
                }
            }
            Some("hit") => {
                // Address
                let metadata = get_address_metadata_from_root(root).ok_or("Invalid address")?;
                let address_owner = metadata.address.trim_start_matches("0x");
                let address_encoded = bs58::encode(address_owner).into_string();
                let timestamp = metadata.timestamp;

                // Table ID
                let input = payload.get("input").ok_or("Invalid field input")?;
                let table_id = input
                    .get("table_id")
                    .ok_or("Invalid field table_id")?
                    .as_str()
                    .ok_or("Invalid table_id")?;

                let mut manager = manager.lock().await;
                let table = manager.get_table_mut(table_id)?;
                let table_id = table.get_id().to_owned();
                let seed = retrieve_seed(timestamp).await?;
                table.hit_player(&address_encoded, timestamp, &seed).await?;

                if !table.any_player_can_hit() {
                    manager.stop_game(&table_id).await?;
                }
            }
            Some("stand") => {
                let input = payload.get("input").ok_or("Invalid field input")?;

                // Parsing JSON
                let game_id = input
                    .get("game_id")
                    .ok_or("Invalid field game_id")?
                    .as_str()
                    .ok_or("Invalid game_id")?;

                let metadata = get_address_metadata_from_root(root).ok_or("Invalid address")?;
                let address_owner = metadata.address.trim_start_matches("0x");
                let address_encoded = bs58::encode(address_owner).into_string();

                let mut manager = manager.lock().await;
                let table = manager.get_table_mut(game_id)?;

                let name = table.get_name_player(&address_encoded).unwrap();
                let table_id = table.get_id().to_owned();
                table.stand_player(&address_encoded, metadata.timestamp)?;

                if !table.any_player_can_hit() {
                    manager.stop_game(&table_id).await?;
                }
                info!("Stand: {} game_id {}", name, game_id);
            }
            _ => Err("Invalid action")?,
        }

        Ok(None)
    }

    async fn load_player_to_mem(
        manager: &Arc<Mutex<Manager>>,
        address_encoded: &str,
    ) -> Result<(), &'static str> {
        let mut manager = manager.lock().await;
        let has_player_in_memory = manager.has_player(address_encoded);
        if !has_player_in_memory {
            let path = get_path_player(address_encoded);
            let player = load_json(&path)
                .await
                .map_err(|_| "Could not load player")?;

            let player = player.as_object().ok_or("Invalid player")?;
            let player_name = player.get("name").ok_or("Invalid field name")?;
            let player_name = player_name.as_str().ok_or("Invalid name")?;

            let player = Player::new(address_encoded.to_string(), player_name.to_string());
            let player = Arc::new(player);
            manager.add_player(player)?;
        }
        Ok(())
    }
}
