pub mod game {
    use crate::{
        models::{
            card::card::Deck,
            player::player::{Player, PlayerHand},
        },
        util::random::generate_id,
    };
    use std::sync::Arc;
    use tokio::sync::Mutex;

    pub struct Manager {
        pub games: Vec<Game>,
        pub players: Vec<Player>,
        pub tables: Vec<Table>,
    }

    impl Default for Manager {
        fn default() -> Self {
            Manager {
                games: Vec::new(),
                tables: Vec::new(),
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
                tables: Vec::with_capacity(game_size),
                players: Vec::new(),
            }
        }

        pub fn add_player(&mut self, player: Player) -> Result<(), &'static str> {
            if self.players.iter().any(|p| p.get_id() == player.get_id()) {
                return Err("Player already registered.");
            }

            self.players.push(player);
            Ok(())
        }

        pub fn remove_player_by_id(&mut self, id: String) -> Result<Player, &'static str> {
            let index = self
                .players
                .iter()
                .position(|player| player.get_id() == id)
                .ok_or("Player not found.")?;
            let player = self.players.remove(index);
            Ok(player)
        }

        pub fn first_game_available(&mut self) -> Result<&mut Game, &'static str> {
            self.games.first_mut().ok_or("No games available.")
        }

        pub fn first_game_available_owned(&mut self) -> Result<Game, &'static str> {
            self.games.pop().ok_or("No games available.")
        }

        pub fn show_games_id_available(&self) -> Vec<String> {
            self.games.iter().map(|game| game.id.clone()).collect()
        }

        pub fn drop_game(&mut self, id: String) -> Result<Game, &'static str> {
            let index = self
                .games
                .iter()
                .position(|game| game.id == id)
                .ok_or("Game not found.")?;
            let game = self.games.swap_remove(index);
            Ok(game)
        }

        /**
         * Players are cleared from the game.
         */
        pub fn realocate_table_to_game(&mut self, table: Table) {
            let mut game = table.game;
            game.players.clear();
            self.games.push(game);
        }

        pub fn add_table(&mut self, table: Table) {
            self.tables.push(table);
        }

        pub fn get_table(&mut self, id: &str) -> Result<&mut Table, &'static str> {
            if self.tables.is_empty() {
                return Err("No tables running.");
            }

            self.tables
                .iter_mut()
                .find(|table| table.game.get_id() == id)
                .ok_or("Table not found or not started.")
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
                id: generate_id(),
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

        // Transforms the game into a table.
        pub fn round_start(self, nth_decks: usize) -> Result<Table, &'static str> {
            if self.players.len() < 2 {
                Err("Minimum number of players not reached.")?;
            }

            Table::new(self, nth_decks)
        }
    }

    /**
     * The table is where the game is played.
     */
    pub struct Table {
        pub deck: Arc<Mutex<Deck>>,
        pub players_with_hand: Vec<PlayerHand>,
        game: Game,
    }

    impl Table {
        fn new(game: Game, nth_decks: usize) -> Result<Self, &'static str> {
            // let bets = Vec::new();
            let mut players_with_hand = Vec::new();
            let deck = Deck::new_with_capacity(nth_decks)?;
            let deck = Arc::new(Mutex::new(deck));

            for player in game.players.iter() {
                let player_hand = PlayerHand::new(
                    player.clone(),
                    deck.clone(),
                    // RefCell::from(ref_table),
                );
                players_with_hand.push(player_hand);
            }

            // @TODO: Implement bet.

            Ok(Table {
                deck,
                players_with_hand,
                game,
            })
        }

        pub fn any_player_can_hit(&self) -> bool {
            self.players_with_hand
                .iter()
                .any(|player| !player.is_standing)
        }

        pub fn find_player_by_id(&mut self, id: &str) -> Result<&mut PlayerHand, &'static str> {
            self.players_with_hand
                .iter_mut()
                .find(|player| player.get_player_id().is_ok_and(|p_id| p_id == id))
                .ok_or("Player not found.")
        }
    }
}
