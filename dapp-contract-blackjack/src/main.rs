use std::{borrow::BorrowMut, env, mem::size_of, sync::Arc};

mod main_test;
mod models;
mod rollups;
mod util;

use dotenv::dotenv;
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

use crate::{
    models::{game::game::Manager, player::player::Player},
    rollups::rollup::rollup,
};

fn get_payload_from_root(root: &Value) -> Option<Value> {
    let root = root.as_object()?;
    let root = root.get("data")?.as_object()?;
    let payload = root.get("payload")?.as_str()?;
    let payload = decode_payload(payload)?;
    Some(payload)
}

fn get_address_metadata_from_root(root: &Value) -> Option<String> {
    let root = root.as_object()?;
    let root = root.get("data")?.as_object()?;
    let metadata = root.get("metadata")?.as_object()?;
    let address = metadata.get("msg_sender")?.as_str()?;
    Some(address.to_owned())
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
            if name.len() > 3 && name.len() < 255 {
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

            let address_owner = get_address_metadata_from_root(root).ok_or("Invalid address")?;
            let address_owner = address_owner.trim_start_matches("0x");
            let address_encoded = bs58::encode(address_owner).into_string();

            // Add player to manager
            let mut manager = manager.lock().await;
            let player = Player::new(address_encoded.clone(), player_name.to_string());
            manager.add_player(player)?;

            if need_write {
                let address_owner_obj = json!({ "address": address_owner });
                let address_path = format!("../data/address/{}.json", address_encoded);

                write_json(&address_path, &address_owner_obj)
                    .await
                    .or(Err("Could not write address"))?;

                let player_path = format!("../data/names/{}.json", encoded_name);
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

            return Ok(Some(response));
        }
        Some("show_games") => {
            let manager = manager.lock().await;
            let games = manager.show_games_id_available();

            let response = generate_message(json!({
                "games": games,
            }));

            return Ok(Some(response));
        }

        Some("start_game") => {
            let input = payload.get("input").ok_or("Invalid field input")?;
            let mut manager = manager.lock().await;
            let game_id = input
                .get("game_id")
                .ok_or("Invalid field game_id")?
                .as_str()
                .ok_or("Invalid game_id")?;

            let game = manager.drop_game(game_id.to_string())?;
            let table = game.round_start(2)?;
            manager.add_table(table);
        }
        Some("hit") => {
            // Address
            let address_owner = get_address_metadata_from_root(root).ok_or("Invalid address")?;
            let address_owner = address_owner.trim_start_matches("0x");
            let address_encoded = bs58::encode(address_owner).into_string();

            // Game ID
            let input = payload.get("input").ok_or("Invalid field input")?;
            let game_id = input
                .get("game_id")
                .ok_or("Invalid field game_id")?
                .as_str()
                .ok_or("Invalid game_id")?;

            let mut manager = manager.lock().await;
            let table = manager.get_table(game_id)?;
            let player = table.find_player_by_id(&address_encoded)?;
            player.hit().await?;
        }
        _ => Err("Invalid action")?,
    }

    Ok(None)
}

async fn handle_game(
    game: Arc<Mutex<Manager>>,
    receiver: &mut Receiver<Value>,
) -> Result<(), &'static str> {
    while let Some(value) = receiver.recv().await {
        println!("Received value: {}", value);
        let _ = handle_request_action(&value, game.clone(), true).await?;
    }

    Ok(())
}

async fn start_listener(game: Arc<Mutex<Manager>>, mut receiver: Receiver<Value>) {
    tokio::spawn(async move {
        while let Err(err) = handle_game(game.clone(), receiver.borrow_mut()).await {
            eprintln!("Listener Error: {}", err);
        }
    });
}

fn start_sender(sender: Sender<Value>) {
    tokio::spawn(async move {
        while let Err(resp) = rollup(&sender).await {
            eprintln!("Sender error: {}", resp);
        }
    });
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let manager = Arc::new(Mutex::new(Manager::default()));
    let (tx, rx) = channel::<Value>(size_of::<Value>());

    env::var("MIDDLEWARE_HTTP_SERVER_URL").expect("Middleware http server must be set");

    start_sender(tx);
    start_listener(manager, rx).await;
}
