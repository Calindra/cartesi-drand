use std::borrow::BorrowMut;
use std::env;
use std::mem::size_of;
use std::sync::Arc;

mod lop;
mod main_test;
mod models;
mod util;

use crate::models::game::game::{Game, Manager};
use dotenv::dotenv;
use serde_json::{json, Map, Value};
use tokio::fs::File;
use tokio::io::{self, AsyncWriteExt};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::Mutex;

use crate::lop::rollup::rollup;
// use crate::models::card::card::{Card, Rank, Suit};
use crate::models::player::player::{Credit, Hand, Player, PlayerBet};

fn get_input_level(obj: &Value) -> Option<&Map<String, Value>> {
    let root = match obj.as_object() {
        Some(root) => root,
        None => return None,
    };

    let input = match root.get("input") {
        Some(input) => match input.as_object() {
            Some(input) => input,
            None => return None,
        },
        None => return None,
    };

    if !input.contains_key("action") {
        return None;
    }

    Some(input)
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
fn is_create_player_action(obj: &Value) -> Option<String> {
    let input = get_input_level(obj)?;

    let is_create_player_action = input
        .get("action")
        .is_some_and(|action| action == "new_player");

    let has_valid_name = input
        .get("name")
        .is_some_and(|name| name.is_string() && name.as_str().unwrap().len() >= 3);

    if is_create_player_action && has_valid_name {
        return Some(input.get("name").unwrap().as_str().unwrap().to_string());
    }

    None
}

async fn handle_game(
    game: Arc<Mutex<Manager>>,
    receiver: &mut Receiver<Value>,
) -> Result<(), &'static str> {
    while let Some(value) = receiver.recv().await {
        println!("Received value: {}", value);

        if let Some(player_name) = is_create_player_action(&value) {
            let encoded_name = bs58::encode(&player_name).into_string();

            let mut manager = game.lock().await;
            let player = Player::new(player_name);
            manager.add_player(player)?;

            let address_owner = "1111111111111111111111111111111";
            let address_encoded = bs58::encode(address_owner).into_string();
            let address_owner_obj = json!({ "address": address_owner });
            let address_path = format!("../data/address/{}.json", address_encoded);

            write_json(&address_path, &address_owner_obj)
                .await
                .or(Err("Could not write address"))?;

            let player_path = format!("../data/names/{}.json", encoded_name);
            let player = json!({ "name": encoded_name });
        }
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

// async fn create_player(player_name: String) -> Player {
//     Player::new(player_name)
// }

#[tokio::main]
async fn main() {
    let example = String::from("hello world");
    let encoded = bs58::encode(example).into_string();
    println!("Encoded: {}", encoded);

    // dotenv().ok();

    // let manager = Arc::new(Mutex::new(Manager::default()));
    // let (tx, rx) = channel::<Value>(size_of::<Value>());

    // env::var("MIDDLEWARE_HTTP_SERVER_URL").expect("Middleware http server must be set");

    // start_sender(tx);
    // start_listener(manager, rx).await;
}
