use std::sync::Arc;

use rand::prelude::*;
use rand_pcg::Pcg64;
use rand_seeder::{Seeder, SipHasher};

use tokio::sync::Mutex;

mod main_test;

#[derive(Debug, Clone)]
enum Suit {
    Spades,   // Espadas
    Hearts,   // Copas
    Diamonds, // Ouros
    Clubs,    // Paus
}

#[derive(Debug, Clone)]
enum Rank {
    Ace = 1,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,  // Valete
    Queen, // Dama
    King,  // Rei
}

#[derive(Debug)]
struct Card {
    suit: Suit,
    rank: Rank,
}

struct Bet {
    amount: u128,
    symbol: String,
}

struct Player {
    name: String,
    hand: Vec<Rank>,
    has_ace: bool,
    bet: Option<Bet>,
}

impl Player {
    fn new(name: String) -> Self {
        Player {
            name,
            hand: Vec::new(),
            has_ace: false,
            bet: None,
        }
    }

    /**
     * Take a card from the deck and add it to the player's hand.
     */
    fn hit() -> Rank {
        let card = Rank::Ace;

        card
    }

    /**
     * Add the value of the card to the player's hand.
     */
    fn stand() {
        todo!();
    }

    /**
     * Double the bet and take one more card.
     */
    fn double_down() {
        todo!();
    }

    /**
     * Split the hand into two separate hands.
     */
    fn split() {
        todo!();
    }

    /**
     * Give up the hand and lose half of the bet.
     */
    fn surrender() {
        todo!();
    }
}

struct Deck {
    cards: Vec<Card>,
}

impl Default for Deck {
    fn default() -> Self {
        let mut cards = Vec::new();

        for suit in [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs].iter() {
            for rank in [
                Rank::Ace,
                Rank::Two,
                Rank::Three,
                Rank::Four,
                Rank::Five,
                Rank::Six,
                Rank::Seven,
                Rank::Eight,
                Rank::Nine,
                Rank::Ten,
                Rank::Jack,
                Rank::Queen,
                Rank::King,
            ]
            .iter()
            {
                cards.push(Card {
                    suit: suit.clone(),
                    rank: rank.clone(),
                });
            }
        }

        Deck { cards }
    }
}

struct Game {
    players: Arc<Mutex<Vec<Player>>>,
    deck: Deck,
    bet: Option<Bet>,
}

impl Game {
    fn new() -> Self {
        Game {
            players: Arc::new(Mutex::new(Vec::new())),
            deck: Deck::default(),
            bet: None,
        }
    }

    fn player_join(&mut self, player: Player) {
        let players = self.players.try_lock();

        let mut players = match players {
            Ok(players) => players,
            Err(_) => return,
        };

        players.push(player);
    }

    fn round_start(self) -> Table {
        Table::new(self)
    }
}

struct Table {
    table: Game,
}

impl Table {
    fn new(table: Game) -> Table {
        Table { table }
    }

    fn get_players(&self) -> &Mutex<Vec<Player>> {
        self.table.players.as_ref()
    }
}

// fn start_game(nth_players: u8) {
//     let mut game = Game::new();

//     for i in 0..nth_players {
//         let player = Player::new(format!("Player {}", i));

//         game.player_join(player);
//     }

//     let table = game.round_start();
// }

fn generate_random_seed(seed: String) -> i32 {
    let mut rng: Pcg64 = Seeder::from(seed).make_rng();
    rng.gen_range(0..51)
}

fn main() {
    println!("Hello, world!");

    // start_game(2);
}
