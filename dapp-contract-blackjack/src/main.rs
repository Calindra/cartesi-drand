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
}

impl Player {
    fn new(name: String) -> Self {
        Player {
            name,
            hand: Vec::new(),
            has_ace: false,
        }
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
}

impl Table {
    fn new() -> Self {
        Table {
            players: Vec::new(),
            deck: Deck::default(),
        }
    }

    fn player_join(&mut self, player: Player) {
        self.players.push(player);
    }
}

fn create_table() {
    let table = Table::new();
}

fn main() {
    println!("Hello, world!");
}
