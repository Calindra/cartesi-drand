use std::env;
use std::error::Error;
use std::mem::size_of;
use std::sync::Arc;

mod lop;
mod main_test;
mod models;
mod util;

use crate::models::game::game::Game;
use dotenv::dotenv;
use serde_json::Value;
use tokio::sync::mpsc::{channel, Receiver};
use tokio::sync::Mutex;

use crate::lop::rollup::rollup;
// use crate::models::card::card::{Card, Rank, Suit};
// use crate::models::player::player::{Credit, Hand, Player, PlayerBet};

async fn handle_game(game: Arc<Mutex<Game>>, mut rx: Receiver<Value>) {
    let _ = tokio::spawn(async move {
        while let Some(value) = rx.recv().await {
            println!("Received value: {}", value);
        }
    }).await;
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let (tx, rx) = channel::<Value>(size_of::<Value>());
    let game = Arc::new(Mutex::new(Game::default()));

    env::var("MIDDLEWARE_HTTP_SERVER_URL").expect("Middleware http server must be set");

    handle_game(game.clone(), rx).await;
    let _ = tokio::spawn(async move {
        rollup(tx).await.unwrap();
    })
    .await;
}
