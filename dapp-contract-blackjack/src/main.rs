use std::env;

mod lop;
mod main_test;
mod models;
mod util;

use dotenv::dotenv;

use crate::lop::rollup::rollup;
// use crate::models::card::card::{Card, Rank, Suit};
// use crate::models::player::player::{Credit, Hand, Player, PlayerBet};

#[tokio::main]
async fn main() {
    dotenv().ok();

    env::var("MIDDLEWARE_HTTP_SERVER_URL").expect("Middleware http server must be set");

    let _ = tokio::spawn(async move {
        rollup().await.unwrap();
    })
    .await;
}
