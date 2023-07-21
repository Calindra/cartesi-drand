use std::{fmt::{Display, self}, sync::Arc, error::Error};

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

impl Card {
    fn show_point(&self) -> u8 {
        let mut point: u8 = self.rank.clone() as u8;

        if self.rank == Rank::Ace {
            point = 11;
        } else if self.rank == Rank::Jack || self.rank == Rank::Queen || self.rank == Rank::King {
            point = 10;
        }

        point
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:} de {:}", &self.rank, &self.suit)
    }
}

struct Credit {
    amount: u32,
    symbol: String,
}

impl Display for Credit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:} {:}", &self.amount, &self.symbol)
    }
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

/**
 * Player registration.
 */
struct Player {
    name: String,
    balance: Option<Credit>,
}

/**
 * Player's hand for specific round while playing.
 */
struct PlayerHand {
    player: Arc<Mutex<PlayerBet>>,
    hand: Hand,
    points: u8,
    is_standing: bool,
    deck: Arc<Mutex<Deck>>,
}

impl Display for PlayerHand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        let player = self.player.try_lock().or(Err(fmt::Error))?;
        let player_name = &player.player.name;
        write!(
            f,
            "{{ name: {:}, points: {:}, hand: {:} }}",
            player_name, &self.points, &self.hand
        )
    }
}

/**
 * Used for the initial of game for bets.
 */
struct PlayerBet {
    player: Player,
    bet: Option<Credit>,
}

impl PlayerBet {
    fn new(name: String) -> PlayerBet {
        PlayerBet {
            player: Player {
                name,
                balance: None,
            },
            bet: None,
        }
    }
}

impl PlayerHand {
    fn new(player: Arc<Mutex<PlayerBet>>, deck: Arc<Mutex<Deck>>) -> PlayerHand {
        PlayerHand {
            player,
            hand: Hand(Vec::new()),
            is_standing: false,
            points: 0,
            deck,
        }
    }

    /**
     * Take a card from the deck and add it to the player's hand.
     */
    fn hit(&mut self) -> Result<(), &'static str> {
        if self.points > 21 {
            return Err("Player is busted.");
        }

        if self.is_standing {
            return Err("Already standing.");
        }

        let mut deck = self.deck.try_lock().or(Err("Error try locking"))?;

        let nth = random::<usize>();
        // let nth = generate_random_seed("blackjack".to_string());
        let size = deck.cards.len();

        let nth = nth % size;

        let card = deck.cards.remove(nth);
        let card_point = card.show_point();
        let points = self.points + card_point;

        if card.rank == Rank::Ace && points > 21 {
            self.points = points - 10;
        } else {
            self.points = points;
        }

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
    fn double_down(&mut self) -> Result<(), &'static str> {
        if self.is_standing {
            return Err("Already standing.");
        }

        let player = self.player.clone();

        let player = player.try_lock().or(Err("Error try locking player"))?;

        let player_balance = player.player.balance.as_ref().ok_or("No balance.")?.amount;
        let player_bet = player.bet.as_ref().ok_or("No bet.")?.amount;

        let double_bet = player_bet.checked_mul(2).ok_or("Could not double bet.")?;

        // self.player.bet.as_mut().and_then(|credit| {
        //     credit.amount = double_bet;
        //     Some(credit)
        // });

        self.hit()?;
        Ok(())
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

impl Deck {
    fn new_with_capacity(nth: usize) -> Result<Self, &'static str> {
        if nth < 1 || nth > 8 {
            eprintln!("Invalid number of decks.");
            Err("Invalid number of decks.")?;
        }

        let mut decks = Deck::default();

        for _ in 1..nth {
            let deck = Deck::default().cards;
            decks.cards.extend(deck);
        }

        Ok(decks)
    }
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

/**
 * This is where the game is initialized.
 */
struct Game {
    players: Vec<Arc<Mutex<PlayerBet>>>,
}

impl Default for Game {
    fn default() -> Self {
        Game {
            players: Vec::new(),
        }
    }
}

impl Game {
    fn player_join(&mut self, player: PlayerBet) -> Result<(), &'static str> {
        if self.players.len() >= 7 {
            return Err("Maximum number of players reached.");
        }

        let player = Arc::new(Mutex::new(player));

        self.players.push(player);
        Ok(())
    }

    fn round_start(&self, nth_decks: usize) -> Result<Table, &'static str> {
        if self.players.len() < 2 {
            panic!("Minimum number of players not reached.");
        }

        Table::new(&self.players, nth_decks)
    }
}

/**
 * The table is where the game is played.
 */
struct Table {
    bets: Vec<Credit>,
    deck: Arc<Mutex<Deck>>,
    players_with_hand: Vec<PlayerHand>,
}

impl Table {
    fn new(players: &Vec<Arc<Mutex<PlayerBet>>>, nth_decks: usize) -> Result<Table, &'static str> {
        let bets = Vec::new();
        let mut players_with_hand = Vec::new();
        let deck = Deck::new_with_capacity(nth_decks)?;
        let deck = Arc::new(Mutex::new(deck));

        for player in players.iter() {
            let player_hand = PlayerHand::new(player.clone(), deck.clone());
            players_with_hand.push(player_hand);
        }

        // @TODO: Implement bet.

        Ok(Table {
            bets,
            deck,
            // players_with_hand: Arc::new(Mutex::new(players_with_hand)),
            players_with_hand,
        })
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
