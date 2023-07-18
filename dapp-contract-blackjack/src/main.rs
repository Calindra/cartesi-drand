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

impl Player {
    fn new(name: String) -> Self {
        Player {
            name,
            hand: Vec::new(),
            has_ace: false,
            bet: 0,
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

struct Player {
    name: String,
    hand: Vec<Rank>,
    has_ace: bool,
    bet: u32,
}

struct Table {
    players: Vec<Player>,
    deck: Deck,
    bet: u32,
}

impl Table {
    fn new() -> Self {
        Table {
            players: Vec::new(),
            deck: Deck::default(),
            bet: 0,
        }
    }

    fn player_join(&mut self, player: Player) {
        self.players.push(player)
    }

    fn round_start(self) -> Game {
        Game::new(self)
    }
}

struct Game {
    table: Table,
}

impl Game {
    fn new(table: Table) -> Game {
        Game { table }
    }
}

fn start_game(nth_players: u8) {
    let mut table = Table::new();

    for i in 0..nth_players {
        let player = Player::new(format!("Player {}", i));

        table.player_join(player);
    }

    let game = table.round_start();
}

fn main() {
    println!("Hello, world!");
}
