pub mod game {
    use crate::models::{
        card::card::Deck,
        player::player::{Credit, Player, PlayerHand},
    };
    use std::{rc::Rc, sync::Arc};
    use tokio::sync::Mutex;
    use uuid::Uuid;

    pub struct Manager {
        pub games: Vec<Game>,
        pub players: Vec<Player>,
    }

    impl Default for Manager {
        fn default() -> Self {
            let games = Vec::new();

            Manager {
                games,
                players: Vec::new(),
            }
        }
    }

    impl Manager {
        pub fn new_with_games(game_size: usize) -> Self {
            let mut games = Vec::with_capacity(game_size);

            for _ in 0..game_size {
                games.push(Game::default());
            }

            Manager {
                games,
                players: Vec::new(),
            }
        }

        pub fn add_player(&mut self, player: Player) -> Result<(), &'static str> {
            self.players.push(player);
            Ok(())
        }
    }

    /**
     * This is where the game is initialized.
     */
    pub struct Game {
        id: String,
        pub players: Vec<Arc<Mutex<Player>>>,
    }

    impl Default for Game {
        fn default() -> Self {
            Game {
                id: Uuid::new_v4().to_string(),
                players: Vec::new(),
            }
        }
    }

    impl Game {
        pub fn get_id(&self) -> &str {
            &self.id
        }

        pub fn player_join(&mut self, player: Player) -> Result<(), &'static str> {
            if self.players.len() >= 7 {
                return Err("Maximum number of players reached.");
            }

            let player = Arc::new(Mutex::new(player));

            self.players.push(player);
            Ok(())
        }

        pub fn round_start(&self, nth_decks: usize) -> Result<Table, &'static str> {
            if self.players.len() < 2 {
                panic!("Minimum number of players not reached.");
            }

            Table::new(&self.players, nth_decks)
        }
    }

    /**
     * The table is where the game is played.
     */
    pub struct Table {
        bets: Vec<Credit>,
        pub deck: Arc<Mutex<Deck>>,
        pub players_with_hand: Vec<PlayerHand>,
    }

    impl Table {
        fn new(players: &Vec<Arc<Mutex<Player>>>, nth_decks: usize) -> Result<Table, &'static str> {
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
                players_with_hand,
            })
        }

        fn any_player_can_hit(&self) -> bool {
            self.players_with_hand
                .iter()
                .any(|player| !player.is_standing)
        }
    }
}
