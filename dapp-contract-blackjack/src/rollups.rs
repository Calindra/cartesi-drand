pub mod rollup {
    use hyper::{
        body::to_bytes, client::HttpConnector, header, Body, Client, Method, Request, Response,
        StatusCode,
    };
    use serde_json::{from_str, json, Value};
    use std::{env, error::Error, str::from_utf8, sync::Arc, time::Duration};
    use tokio::sync::{mpsc::Sender, Mutex};

    use crate::{
        models::{
            game::game::Manager,
            player::{check_fields_create_player, player::Player},
        },
        util::json::{
            decode_payload, generate_message, get_address_metadata_from_root, write_json,
        },
    };

    pub async fn rollup(
        manager: Arc<Mutex<Manager>>,
        sender: &Sender<Value>,
    ) -> Result<(), Box<dyn Error>> {
        println!("Starting loop...");

        let client = Client::new();
        let server_addr = env::var("MIDDLEWARE_HTTP_SERVER_URL")?;

        let mut status = "accept";
        loop {
            println!("Sending finish");
            let response = json!({ "status": status.clone() });
            let request = Request::builder()
                .method(Method::POST)
                .header(header::CONTENT_TYPE, "application/json")
                .uri(format!("{}/finish", &server_addr))
                .body(Body::from(response.to_string()))?;
            let response = client.request(request).await?;
            let status_response = response.status();
            println!("Receive finish status {}", &status_response);

            if status_response == StatusCode::ACCEPTED {
                println!("No pending rollup request, trying again");
            } else {
                let body = to_bytes(response).await?;
                let body = from_utf8(&body)?;
                let body = from_str::<Value>(body)?;

                let request_type = body["request_type"]
                    .as_str()
                    .ok_or("request_type is not a string")?;

                status = match request_type {
                    "advance_state" => {
                        handle_advance(manager.clone(), &client, &server_addr[..], body, sender)
                            .await?
                    }
                    "inspect_state" => {
                        handle_inspect(manager.clone(), &client, &server_addr[..], body, sender)
                            .await?
                    }
                    &_ => {
                        eprintln!("Unknown request type");
                        "reject"
                    }
                }
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    async fn handle_inspect(
        manager: Arc<Mutex<Manager>>,
        client: &Client<HttpConnector>,
        server_addr: &str,
        body: Value,
        sender: &Sender<Value>,
    ) -> Result<&'static str, Box<dyn Error>> {
        println!("Handling inspect");

        println!("body {:}", &body);

        // sender.send(body).await?;
        // handle_request_action(&body, manager, false).await?;

        let payload = get_payload_from_root(&body).ok_or("Invalid payload")?;
        let action = get_from_payload_action(&payload);
        match action.as_deref() {
            Some("show_games") => {
                let manager = manager.lock().await;
                let games = manager.show_games_id_available();

                let response = json!({
                    "games": games,
                });
                let json_as_hex = hex::encode(response.to_string());
                let report = json!({ "payload": format!("0x{}", json_as_hex) });
                println!("Report: {:}", report);
                let _ = send_report(report.clone()).await;
            }
            Some("show_hands") => {
                let input = payload.get("input").ok_or("Invalid field input")?;

                // Parsing JSON
                let game_id = input
                    .get("game_id")
                    .ok_or("Invalid field game_id")?
                    .as_str()
                    .ok_or("Invalid game_id")?;

                let mut manager = manager.lock().await;

                let table = manager.get_table(game_id)?;
                let hands = table.generate_hands();
                let json_as_hex = hex::encode(hands.to_string());
                let report = json!({ "payload": format!("0x{}", json_as_hex) });

                println!("Report: {:}", report);
                let _ = send_report(report.clone()).await;
            }
            Some("show_winner") => {
                let input = payload.get("input").ok_or("Invalid field input")?;

                // Parsing JSON
                let game_id = input
                    .get("game_id")
                    .ok_or("Invalid field game_id")?
                    .as_str()
                    .ok_or("Invalid game_id")?;

                let table_id = input
                    .get("table_id")
                    .ok_or("Invalid field table_id")?
                    .as_str()
                    .ok_or("Invalid string table_id")?;

                let manager = manager.lock().await;

                println!(
                    "Finding score by table_id {} and game_id {} ...",
                    table_id, game_id
                );
                let scoreboard = manager
                    .get_scoreboard(table_id, game_id)
                    .ok_or("Scoreboard not found searching by table_id")?;
                
                let response = json!({
                    "scoreboard": scoreboard.to_json(),
                });
                let json_as_hex = hex::encode(response.to_string());
                let report = json!({ "payload": format!("0x{}", json_as_hex) });

                println!("Report: {:}", report);
                let _ = send_report(report.clone()).await;
            }
            _ => Err("Invalid inspect")?,
        };

        Ok("accept")
    }

    async fn handle_advance(
        manager: Arc<Mutex<Manager>>,
        client: &Client<HttpConnector>,
        server_addr: &str,
        body: Value,
        sender: &Sender<Value>,
    ) -> Result<&'static str, Box<dyn Error>> {
        println!("Handling advance");

        println!("body {:}", &body);

        // sender.send(body).await?;
        let payload = get_payload_from_root(&body).ok_or("Invalid payload")?;
        let action = get_from_payload_action(&payload);

        match action.as_deref() {
            Some("new_player") => {
                let input = payload.get("input").ok_or("Invalid field input")?;
                let player_name = check_fields_create_player(&input)?;

                let encoded_name = bs58::encode(&player_name).into_string();

                let metadata = get_address_metadata_from_root(&body).ok_or("Invalid address")?;
                let address_owner = metadata.address.trim_start_matches("0x");
                let address_encoded = bs58::encode(address_owner).into_string();

                // Add player to manager
                let player = Player::new(address_encoded.clone(), player_name.to_string());
                let mut manager = manager.lock().await;
                let player = Arc::new(player);
                manager.add_player(player)?;

                // Persist player
                let need_write = true;
                if need_write {
                    let address_owner_obj = json!({ "address": address_owner });
                    let address_path = format!("./data/address/{}.json", address_encoded);

                    write_json(&address_path, &address_owner_obj)
                        .await
                        .or(Err("Could not write address"))?;

                    let player_path = format!("./data/names/{}.json", encoded_name);
                    let player = json!({ "name": encoded_name });
                    write_json(&player_path, &player)
                        .await
                        .or(Err("Could not write player"))?;
                }

                let response = generate_message(json!({
                    "address": address_encoded,
                    "encoded_name": encoded_name,
                    "name": player_name,
                }));

                println!("Response: {:}", response);

                return Ok("accept");
            }
            Some("join_game") => {
                let input = payload.get("input").ok_or("Invalid field input")?;

                // Address
                let metadata = get_address_metadata_from_root(&body).ok_or("Invalid address")?;
                let address_owner = metadata.address.trim_start_matches("0x");
                let address_encoded = bs58::encode(address_owner).into_string();

                let mut manager = manager.lock().await;
                let player = manager.get_player_ref(&address_encoded)?;

                // Parsing JSON
                let game_id = input
                    .get("game_id")
                    .ok_or("Invalid field game_id")?
                    .as_str()
                    .ok_or("Invalid game_id")?;

                manager.player_join(game_id, player.clone())?;
                println!("Player joined: name {} game_id {}", player.name, game_id);
            }

            Some("start_game") => {
                let input = payload.get("input").ok_or("Invalid field input")?;
                let metadata = get_address_metadata_from_root(&body).ok_or("Invalid address")?;
                // Parsing JSON
                let game_id = input
                    .get("game_id")
                    .ok_or("Invalid field game_id")?
                    .as_str()
                    .ok_or("Invalid game_id")?;

                let mut manager = manager.lock().await;

                // Get game and make owner
                let game = manager.drop_game(game_id)?;
                // Generate table from game
                let table = game.round_start(2, metadata.timestamp)?;
                // Add table to manager
                manager.add_table(table);
                println!("Game started: game_id {}", game_id);
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

            Some("hit") => {
                // Address
                let metadata = get_address_metadata_from_root(&body).ok_or("Invalid address")?;
                let address_owner = metadata.address.trim_start_matches("0x");
                let address_encoded = bs58::encode(address_owner).into_string();
                let timestamp = metadata.timestamp;

                // Game ID
                let input = payload.get("input").ok_or("Invalid field input")?;
                let game_id = input
                    .get("game_id")
                    .ok_or("Invalid field game_id")?
                    .as_str()
                    .ok_or("Invalid game_id")?;

                let mut manager = manager.lock().await;
                let table = manager.get_table(game_id)?;
                let table_id = table.get_id().to_owned();
                table.hit_player(&address_encoded, timestamp).await?;

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

                let metadata = get_address_metadata_from_root(&body).ok_or("Invalid address")?;
                let address_owner = metadata.address.trim_start_matches("0x");
                let address_encoded = bs58::encode(address_owner).into_string();

                let mut manager = manager.lock().await;
                let table = manager.get_table(game_id)?;

                let name = table.get_name_player(&address_encoded).unwrap();
                let table_id = table.get_id().to_owned();
                table.stand_player(&address_encoded, metadata.timestamp)?;

                if !table.any_player_can_hit() {
                    manager.stop_game(&table_id).await?;
                }
                println!("Stand: {} game_id {}", name, game_id);
            }
            _ => Err("Invalid action")?,
        }

        Ok("accept")
    }

    pub(crate) async fn send_report(
        report: Value,
    ) -> Result<&'static str, Box<dyn std::error::Error>> {
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
}
