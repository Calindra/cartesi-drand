use std::{env, mem::size_of, sync::Arc};

mod models;
mod rollups;
mod util;

use dotenv::dotenv;
use rollups::rollup::{rollup, send_report};
use serde_json::{json, Value};
use tokio::{
    fs::File,
    io::{self, AsyncWriteExt},
    sync::{
        mpsc::{channel, Receiver, Sender},
        Mutex,
    },
};
use util::json::{decode_payload, generate_message};

use crate::models::{game::game::Manager, player::player::Player};

struct Metadata {
    address: String,
    timestamp: u64,
    // input_index: u64,
}

fn get_payload_from_root(root: &Value) -> Option<Value> {
    let root = root.as_object()?;
    let root = root.get("data")?.as_object()?;
    let payload = root.get("payload")?.as_str()?;
    let payload = decode_payload(payload)?;
    Some(payload)
}

fn get_address_metadata_from_root(root: &Value) -> Option<Metadata> {
    let root = root.as_object()?;
    let root = root.get("data")?.as_object()?;
    let metadata = root.get("metadata")?.as_object()?;

    let address = metadata.get("msg_sender")?.as_str()?;
    let timestamp = metadata.get("timestamp")?.as_u64()?;
    // let input_index = metadata.get("input_index")?.as_u64()?;

    Some(Metadata {
        address: address.to_owned(),
        timestamp,
        // input_index,
    })
}

fn get_from_payload_action(payload: &Value) -> Option<String> {
    let input = payload.get("input")?.as_object()?;
    let action = input.get("action")?.as_str()?;
    Some(action.to_owned())
}

async fn write_json(path: &str, obj: &Value) -> Result<(), io::Error> {
    let mut file = File::create(path).await?;
    let value = obj.to_string();
    file.write_all(value.as_bytes()).await?;
    Ok(())
}

/**
 * Example of call:
 * {"input":{"name":"Bob","action":"new_player"}}
 */
fn check_fields_create_player(input: &Value) -> Result<&str, &'static str> {
    input
        .get("name")
        .ok_or("Field name dont exist")?
        .as_str()
        .ok_or("Field name isnt a string")
        .and_then(|name| {
            if name.len() >= 3 && name.len() <= 255 {
                Ok(name)
            } else {
                Err("Name need between 3 and 255 characters")
            }
        })
}

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

            // Add player to manager
            let player = Player::new(address_encoded.clone(), player_name.to_string());
            let mut manager = manager.lock().await;
            let player = Arc::new(player);
            manager.add_player(player)?;

            // Persist player
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

            return Ok(Some(response));
        }
        Some("join_game") => {
            let input = payload.get("input").ok_or("Invalid field input")?;

            // Address
            let metadata = get_address_metadata_from_root(root).ok_or("Invalid address")?;
            let address_owner = metadata.address.trim_start_matches("0x");
            let address_encoded = bs58::encode(address_owner).into_string();

            let mut manager = manager.lock().await;
            let player = manager.get_player_ref(address_encoded.clone())?;

            // Parsing JSON
            let game_id = input
                .get("game_id")
                .ok_or("Invalid field game_id")?
                .as_str()
                .ok_or("Invalid game_id")?;

            let game = manager.get_game_by_id(game_id.to_string())?;
            game.player_join(player.clone())?;
        }
        Some("show_games") => {
            let manager = manager.lock().await;
            let games = manager.show_games_id_available();

            let response = generate_message(json!({
                "games": games,
            }));

            println!("Response: {:}", response);
            let _ = send_report(response.clone()).await;
            return Ok(Some(response));
        }

        Some("start_game") => {
            let input = payload.get("input").ok_or("Invalid field input")?;

            // Parsing JSON
            let game_id = input
                .get("game_id")
                .ok_or("Invalid field game_id")?
                .as_str()
                .ok_or("Invalid game_id")?;

            let mut manager = manager.lock().await;

            // Get game and make owner
            let game = manager.drop_game(game_id.to_string())?;
            // Generate table from game
            let table = game.round_start(2)?;
            // Add table to manager
            manager.add_table(table);
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
            table.hit_player(&address_encoded, timestamp).await?;
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
            let player = table.find_player_by_id(&address_encoded)?;

            player.stand().await?;
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
            println!("Received value: {}", value);
            todo!();
        }
    });
}

#[tokio::main]
async fn main() {
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
