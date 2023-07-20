use std::{sync::Arc, fmt::Display};

use rand::prelude::*;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;

use tokio::sync::Mutex;

mod main_test;

#[derive(Clone)]
enum Suit {
    Spades,   // Espadas
    Hearts,   // Copas
    Diamonds, // Ouros
    Clubs,    // Paus
}

impl Display for Suit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let suit = match self {
            Suit::Spades => "Espadas",
            Suit::Hearts => "Copas",
            Suit::Diamonds => "Ouros",
            Suit::Clubs => "Paus",
        };

        write!(f, "{}", suit)
    }
}

#[derive(Clone, PartialEq)]
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

impl Display for Rank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rank = self.clone() as u8;
        let rank_name: String = {
            if rank > 1 && rank < 11 {
                rank.to_string()
            } else {
                match self {
                    Rank::Ace => "Ás".to_string(),
                    Rank::Jack => "Valete".to_string(),
                    Rank::Queen => "Dama".to_string(),
                    Rank::King => "Rei".to_string(),
                    _ => "".to_string(),
                }
            }
        };

        write!(f, "{}", rank_name)
    }
}

struct Card {
    suit: Suit,
    rank: Rank,
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:} de {:}", &self.rank, &self.suit)
    }
}

struct Bet {
    amount: u128,
    symbol: String,
}

struct Hand(pub Vec<Card>);

impl Display for Hand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        let _ = self.0.iter().fold(Ok(()), |result, el| {
            result.and_then(|_| write!(f, " {},", &el))
        });
        write!(f, " ]")
    }
}

struct Player {
    name: String,
    hand: Hand,
    has_ace: bool,
    bet: Option<Bet>,
    is_standing: bool,
}

impl Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ name: {:}, hand: {:} }}", &self.name, &self.hand)
    }
}

impl Player {
    fn new(name: String) -> Self {
        Player {
            name,
            hand: Hand(Vec::new()),
            has_ace: false,
            bet: None,
            is_standing: false,
        }
    }

    /**
     * Take a card from the deck and add it to the player's hand.
     */
    fn hit(&mut self, deck: &mut Deck) -> Result<(), &'static str> {
        if self.is_standing {
            return Err("Already standing.");
        }

        let nth = random::<usize>();
        // let nth = generate_random_seed("blackjack".to_string());
        let size = deck.cards.len();

        let nth = nth % size;

        let card = deck.cards.remove(nth);

        self.has_ace = self.has_ace || card.rank == Rank::Ace;
        self.hand.0.push(card);

        Ok(())
    }

    /**
     * Add the value of the card to the player's hand.
     */
    fn stand(&mut self) -> Result<(), ()> {
        self.is_standing = true;
        Ok(())
    }

    /**
     * Double the bet and take one more card.
     */
    fn double_down(&mut self, deck: &mut Deck) -> Result<(), &'static str> {
        if self.is_standing {
            return Err("Already standing.");
        }
        if let Some(bet) = self.bet.as_ref() {
            self.bet = Some(Bet {
                amount: bet.amount * 2,
                symbol: bet.symbol.clone(),
            });

            let is_err = self.hit(deck).is_err();

            if is_err {
                return Err("Could not hit.");
            }

            return Ok(());
        }

        Err("No bet.")
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
    deck: Arc<Mutex<Deck>>,
}

impl Table {
    fn new(table: Game) -> Table {
        Table {
            game: table,
            deck: Arc::new(Mutex::new(Deck::default())),
        }
    }

    fn get_players(&self) -> Arc<Mutex<Vec<Player>>> {
        self.game.players.clone()
    }
}

fn generate_random_seed(seed: String) -> usize {
    let mut rng: Pcg64 = Seeder::from(seed).make_rng();
    rng.gen_range(0..51)
}

fn main() {
    println!("Hello, world!");

    // start_game(2);
}
