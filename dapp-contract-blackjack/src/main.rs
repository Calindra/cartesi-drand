use std::{
    fmt::{self, Display},
    sync::Arc,
};

mod main_test;
mod models;
mod util;

use crate::models::card::card::{Card, Rank, Suit};
use crate::models::player::player::{Credit, Hand, Player, PlayerBet};

fn main() {
    println!("Hello, world!");
}
