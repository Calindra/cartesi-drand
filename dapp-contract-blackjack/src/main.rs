use std::{
    collections::{BTreeMap, BinaryHeap},
    fmt::{self, Display},
    sync::Arc,
};

mod lop;
mod main_test;
mod models;
mod util;

use dotenv::dotenv;

use crate::lop::rollup::rollup;
use crate::models::card::card::{Card, Rank, Suit};
use crate::models::player::player::{Credit, Hand, Player, PlayerBet};

#[tokio::main]
async fn main() {
    // dotenv().unwrap();

    println!("Hello world");

    let mut map = BTreeMap::new();

    map.insert(2, "b");
    map.insert(1, "a");
    map.insert(3, "c");

    let s = map.range(1..).next().unwrap();


    assert_eq!(*s.0, 1);

    // rollup().await.unwrap();
}
