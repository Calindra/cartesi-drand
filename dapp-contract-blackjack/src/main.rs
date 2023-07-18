enum Card {
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

struct Player {
    name: String,
    hand: Vec<Card>,
    has_ace: bool,
    bet: u32,
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
    fn hit() -> Card {
        let card = Card::Ace;

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
    kind: Vec<Card>,
}

impl Default for Deck {
    fn default() -> Self {
        let deck = Deck { kind: Vec::new() };

        deck
    }
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
