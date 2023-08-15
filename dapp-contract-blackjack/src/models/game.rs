pub mod game {
    use crate::{
        models::{
            card::card::Deck,
            player::player::{Player, PlayerHand},
        },
        util::random::generate_id,
    };
    use serde_json::json;
    use std::{collections::HashMap, sync::Arc};
    use tokio::sync::Mutex;

    pub struct Manager {
        pub games: Vec<Game>, // games to be started. A player can join this game
        pub players: HashMap<String, Arc<Player>>,
        pub tables: Vec<Table>, // games running
    }

    impl Default for Manager {
        fn default() -> Self {
            Manager {
                games: Vec::new(),
                tables: Vec::new(),
                players: HashMap::new(),
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
                players: HashMap::new(),
            }
        }

        pub fn add_player(&mut self, player: Arc<Player>) -> Result<(), &'static str> {
            if self.players.contains_key(&player.get_id()) {
                return Err("Player already registered.");
            }

            self.players.insert(player.get_id(), player);
            Ok(())
        }

        pub fn remove_player_by_id(&mut self, id: String) -> Result<Arc<Player>, &'static str> {
            let player = self.players.remove(&id).ok_or("Player not found.")?;
            Ok(player)
        }

        pub fn get_player_ref(&mut self, address: String) -> Result<Arc<Player>, &'static str> {
            let player = self.remove_player_by_id(address)?;
            self.players.insert(player.get_id(), player.clone());
            Ok(player)
        }

        pub fn first_game_available(&mut self) -> Result<&mut Game, &'static str> {
            self.games.first_mut().ok_or("No games available.")
        }

        pub fn get_game_by_id(&mut self, id: String) -> Result<&mut Game, &'static str> {
            self.games
                .iter_mut()
                .find(|game| game.id == id)
                .ok_or("Game not found.")
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
        pub fn reallocate_table_to_game(&mut self, table: Table) {
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

        pub fn get_table_owned(&mut self, id: &str) -> Result<Table, &'static str> {
            let index = self
                .tables
                .iter_mut()
                .position(|table| table.game.get_id() == id)
                .ok_or("Table not found or not started.")?;
            let table = self.tables.swap_remove(index);
            Ok(table)
        }

        pub fn player_join(
            &mut self,
            game_id: String,
            player: Arc<Player>,
        ) -> Result<(), &'static str> {
            if !self.players.contains_key(&player.get_id()) {
                // self.add_player(player.clone())?;
                return Err("Player isnt not registered");
            }

            let game = self.get_game_by_id(game_id)?;

            if game.players.len() >= 7 {
                return Err("Maximum number of players reached.");
            }

            if game.players.iter().any(|p| p.get_id() == player.get_id()) {
                return Err("Player already registered.");
            }

            game.players.push(player);
            Ok(())
        }
    }

    /**
     * The scoreboard is where the game is finished.
     */
    pub struct Scoreboard {
        game_id: String,
        players: Vec<Arc<Player>>,
        winner: Option<Arc<Player>>,
    }

    /**
     * This is where the game is initialized.
     */
    pub struct Game {
        id: String,
        pub players: Vec<Arc<Player>>,
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

        // Transforms the game into a table.
        pub fn round_start(self, nth_decks: usize, last_timestamp: u64) -> Result<Table, &'static str> {
            if self.players.len() < 2 {
                Err("Minimum number of players not reached.")?;
            }

            Table::new(self, nth_decks, last_timestamp)
        }
    }

    /**
     * The table is where the game is played.
     */
    pub struct Table {
        pub deck: Arc<Mutex<Deck>>,
        pub players_with_hand: Vec<PlayerHand>,
        game: Game,
        round: u8,
    }

    impl Table {
        pub fn get_round(&self) -> u8 {
            self.round
        }

        fn new(game: Game, nth_decks: usize, last_timestamp: u64) -> Result<Self, &'static str> {
            // let bets = Vec::new();
            let players_with_hand = Vec::new();
            let deck = Deck::new_with_capacity(nth_decks).map(|deck| Arc::new(Mutex::new(deck)))?;

            let mut table = Self {
                deck,
                players_with_hand,
                game,
                round: 1,
            };

            table.game.players.iter().for_each(|player| {
                let player = player.clone();
                let player_hand = PlayerHand::new(player, table.deck.clone(), last_timestamp);
                table.players_with_hand.push(player_hand);
            });

            // @TODO: Implement bet.

            Ok(table)
        }

        pub async fn hit_player(
            &mut self,
            player_id: &str,
            timestamp: u64,
        ) -> Result<(), &'static str> {
            let round = self.round;
            let player = self.find_player_by_id(player_id)?;
            let player_round = player.get_round();
            if round != player.get_round() {
                println!("Game round {}; Player round {}; Player id {};", round, player_round, player_id);
                Err("Round is not the same. Waiting for another players.")?;
            }

            player.hit(timestamp).await?;
            player.last_timestamp = timestamp;

            self.next_round();

            Ok(())
        }

        fn next_round(&mut self) {
            if self.any_player_can_hit() {
                return;
            }

            self.round = self.round + 1;
        }

        pub fn any_player_can_hit(&self) -> bool {
            self.players_with_hand
                .iter()
                .any(|player| !player.is_standing && self.round == player.get_round())
        }

        pub fn find_player_by_id(&mut self, id: &str) -> Result<&mut PlayerHand, &'static str> {
            self.players_with_hand
                .iter_mut()
                .find(|player| player.get_player_id().is_ok_and(|p_id| p_id == id))
                .ok_or("Player not found.")
        }

        pub fn generate_hands(&self) -> serde_json::Value {
            json!({
                "players": self.players_with_hand.iter().map(|player| player.generate_hand()).collect::<Vec<serde_json::Value>>()
            })
        }
    }
}
