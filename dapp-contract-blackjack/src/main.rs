use std::sync::Arc;

use rand::prelude::*;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;

use tokio::sync::Mutex;

mod main_test;

#[derive(Debug, Clone)]
enum Suit {
    Spades,   // Espadas
    Hearts,   // Copas
    Diamonds, // Ouros
    Clubs,    // Paus
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone)]
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
    hand: Arc<Mutex<Vec<Card>>>,
    has_ace: bool,
    bet: Option<Bet>,
    is_standing: bool,
}

impl Player {
    fn new(name: String) -> Self {
        Player {
            name,
            hand: Arc::new(Mutex::new(Vec::new())),
            has_ace: false,
            bet: None,
            is_standing: false,
        }
    }

    /**
     * Take a card from the deck and add it to the player's hand.
     */
    fn hit(&mut self, deck: &Deck) -> Option<usize> {
        if self.is_standing {
            return None;
        }

        let nth = random::<usize>();
        let size = deck.cards.len();

        let nth = nth % size;

        let card = deck.cards[nth].clone();

        let hand = self.hand.try_lock();

        if let Ok(mut hand) = hand {
            hand.push(card.clone());
            self.has_ace = self.has_ace || card.rank == Rank::Ace;

            return Some(nth);
        }

        None
    }

    /**
     * Add the value of the card to the player's hand.
     */
    fn stand(&mut self) {
        self.is_standing = true;
    }

    /**
     * Double the bet and take one more card.
     */
    fn double_down(&mut self, deck: &Deck) -> bool {
        if self.is_standing {
            return false;
        }
        if let Some(bet) = self.bet.as_ref() {
            self.bet = Some(Bet {
                amount: bet.amount * 2,
                symbol: bet.symbol.clone(),
            });

            let is_hit = self.hit(deck).is_some();

            return is_hit;
        }

        false
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

    bet: Option<Bet>,
}

/**
 * This is where the game is initialized.
 */
impl Game {
    fn new() -> Self {
        Game {
            players: Arc::new(Mutex::new(Vec::new())),

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

/**
 * The table is where the game is played.
 */
struct Table {
    game: Game,
    deck: Deck,
}

impl Table {
    fn new(table: Game) -> Table {
        Table {
            game: table,
            deck: Deck::default(),
        }
    }

    fn get_players(&self) -> Arc<Mutex<Vec<Player>>> {
        self.game.players.clone()
    }
}

fn generate_random_seed(seed: String) -> i32 {
    let mut rng: Pcg64 = Seeder::from(seed).make_rng();
    rng.gen_range(0..51)
}

fn main() {
    println!("Hello, world!");

    // start_game(2);
}
