use std::{env, mem::size_of, sync::Arc};

mod models;
mod rollups;
mod util;

use dotenv::dotenv;
use rollups::rollup::{get_from_payload_action, get_payload_from_root, rollup, send_report};
use serde_json::{json, Value};
use tokio::sync::{
    mpsc::{channel, Receiver, Sender},
    Mutex,
};
use util::json::generate_message;

use crate::{
    models::{
        game::game::{Manager, Scoreboard, Table, TableJson},
        player::{check_fields_create_player, player::Player},
    },
    util::json::{get_address_metadata_from_root, write_json},
};

pub async fn handle_request_action(
    root: &Value,
    manager: Arc<Mutex<Manager>>,
    need_write: bool,
) -> Result<Option<Value>, &'static str> {
    let payload = get_payload_from_root(root).ok_or("Invalid payload")?;
    let action = get_from_payload_action(&payload);

    match action.as_deref() {
        Some("new_player") => {
            let input = payload.get("input").ok_or("Invalid field input")?;
            let player_name = check_fields_create_player(&input)?;

            let encoded_name = bs58::encode(&player_name).into_string();

            let metadata = get_address_metadata_from_root(root).ok_or("Invalid address")?;
            let address_owner = metadata.address.trim_start_matches("0x");
            let address_encoded = bs58::encode(address_owner).into_string();

            // Persist player
            if need_write {
                let address_owner_obj = json!({ "address": address_owner, "name": player_name });
                let address_path = format!("./data/address/{}.json", address_encoded);

                write_json(&address_path, &address_owner_obj)
                    .await
                    .or(Err("Could not write address"))?;

                let player_path = format!("./data/names/{}.json", encoded_name);
                let player = json!({ "name": encoded_name, "address": metadata.address });
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

            return Ok(Some(response));
        }
        Some("join_game") => {
            let input = payload.get("input").ok_or("Invalid field input")?;

            // Address
            let metadata = get_address_metadata_from_root(root).ok_or("Invalid address")?;
            let address_owner = metadata.address.trim_start_matches("0x");
            let address_encoded = bs58::encode(address_owner).into_string();

            let mut manager = manager.lock().await;
            let player = manager.get_player_by_b58_address(&address_encoded)?;

            // Parsing JSON
            let game_id = input
                .get("game_id")
                .ok_or("Invalid field game_id")?
                .as_str()
                .ok_or("Invalid game_id")?;

            manager.player_join(game_id, player.clone())?;
            println!("Player joined: name {} game_id {}", player.name, game_id);
        }
        Some("show_games") => {
            let manager = manager.lock().await;
            let games = manager.show_games_id_available();

            let response = generate_message(json!({
                "games": games,
            }));

            println!("Response: {:}", response);

            return Ok(Some(response));
        }

        Some("start_game") => {
            let input = payload.get("input").ok_or("Invalid field input")?;
            let metadata = get_address_metadata_from_root(root).ok_or("Invalid address")?;
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
            let players_with_hand = table.players_with_hand.to_vec();
            // Add table to manager
            manager.add_table(table);
            let timestamp = metadata.timestamp;
            for _ in 0..2 {
                for ph in players_with_hand.iter() {
                    let table = manager.get_table(game_id).unwrap();
                    table.hit_player(&ph.get_player_id(), timestamp).await?;
                    table.any_player_can_hit();
                }
            }

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
            let response = generate_message(hands);

            println!("Response: {:}", response);

            return Ok(Some(response));
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
            let scoreboard = manager.get_scoreboard(table_id, game_id);
            let scoreboard = match scoreboard {
                Some(scoreboard) => scoreboard.to_json(),
                None => {
                    let json = manager.get_json_table_by_id(table_id).unwrap();
                    let table_json = serde_json::from_str::<TableJson>(&json).unwrap();
                    let scoreboard = table_json.get_scoreboard().await;
                    scoreboard.to_json()
                }
            };
            println!("Response: {:}", scoreboard);
            let response = generate_message(scoreboard);
            let response = generate_message(json!(response));
            return Ok(Some(response));
        }

        Some("hit") => {
            // Address
            let metadata = get_address_metadata_from_root(root).ok_or("Invalid address")?;
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

            let metadata = get_address_metadata_from_root(root).ok_or("Invalid address")?;
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

    Ok(None)
}

fn start_listener(
    manager: Arc<Mutex<Manager>>,
    mut receiver: Receiver<Value>,
    sender_middleware: Sender<Value>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            let receive = receiver.recv().await;

            if let Some(value) = receive {
                println!("Received value: {}", value);

                // @todo need return responses
                let value = handle_request_action(&value, manager.clone(), true)
                    .await
                    .map_err(|err| {
                        eprintln!("Listener Error: {}", err);
                        err
                    });

                if let Ok(Some(value)) = value {
                    let _ = sender_middleware.send(value).await.map_err(|err| {
                        eprintln!("Send to middleware error: {}", err);
                        err
                    });
                }
            }
        }
    })
}

fn start_sender(manager: Arc<Mutex<Manager>>, sender: Sender<Value>) {
    tokio::spawn(async move {
        loop {
            if let Err(resp) = rollup(manager.clone(), &sender).await {
                eprintln!("Sender error: {}", resp);
            }
        }
    });
}

fn listener_send_message_to_middleware(mut receiver: Receiver<Value>) {
    tokio::spawn(async move {
        while let Some(value) = receiver.recv().await {
            println!("Send value to middleware: {}", value);
            let _ = send_report(value).await;
        }
    });
}

#[tokio::main]
async fn main() {
    const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");
    println!("BlackJack v{}", VERSION.unwrap_or("unknown"));

    dotenv().ok();
    env::var("MIDDLEWARE_HTTP_SERVER_URL").expect("Middleware http server must be set");

    const SLOTS: usize = 10;

    let manager = Arc::new(Mutex::new(Manager::new_with_games(SLOTS)));
    let (sender_rollup, receiver_rollup) = channel::<Value>(size_of::<Value>());
    let (sender_middl, receiver_middl) = channel::<Value>(size_of::<Value>());

    start_sender(manager.clone(), sender_rollup);
    listener_send_message_to_middleware(receiver_middl);
    let _ = start_listener(manager, receiver_rollup, sender_middl).await;
}
