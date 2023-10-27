pub mod game {
    use crate::{
        models::{
            card::card::Deck,
            player::player::{Player, PlayerHand},
        },
        util::{json::generate_report, random::generate_id},
    };
    use log::{debug, info};
    use serde_json::{json, Value};
    use std::{collections::HashMap, sync::Arc};
    use tokio::sync::Mutex;

    pub struct Manager {
        pub games: Vec<Game>, // games to be started. A player can join this game
        pub players: HashMap<String, Arc<Player>>,
        pub tables: HashMap<String, Table>, // games running
        scoreboards: Vec<Scoreboard>,
        pub games_report_cache: Option<Value>,
    }

    impl Default for Manager {
        fn default() -> Self {
            Manager {
                games: Vec::new(),
                tables: HashMap::new(),
                players: HashMap::new(),
                scoreboards: Vec::new(),
                games_report_cache: None,
            }
        }
    }

    impl Manager {
        pub fn new_with_games(game_size: usize) -> Self {
            let mut games = Vec::with_capacity(game_size);

            for i in 1..=game_size {
                let id = i.to_string();
                let game = Game::with_id(id);
                games.push(game);
            }

            let report = Manager::generate_games_report(&games);

            Manager {
                games,
                tables: HashMap::with_capacity(game_size),
                players: HashMap::new(),
                scoreboards: Vec::new(),
                games_report_cache: Some(report),
            }
        }

        pub fn generate_games_report(games: &Vec<Game>) -> Value {
            let games = games
                .iter()
                .map(|game| {
                    json!({
                        "id": game.get_id(),
                        "players": game.players.len(),
                    })
                })
                .collect::<Vec<_>>();

            generate_report(json!({
                "games": games,
            }))
        }

        pub fn add_player(&mut self, player: Arc<Player>) -> Result<(), &'static str> {
            if self.players.contains_key(&player.get_id()) {
                return Err("Player already registered.");
            }

            self.players.insert(player.get_id(), player);
            Ok(())
        }

        pub fn has_player(&self, id: &str) -> bool {
            self.players.contains_key(id)
        }

        pub fn remove_player_by_id(&mut self, id: &str) -> Result<Arc<Player>, &'static str> {
            let player = self.players.remove(id).ok_or("Player not found.")?;
            Ok(player)
        }

        pub fn get_player_ref(&mut self, address: &str) -> Result<Arc<Player>, &'static str> {
            let player = self.remove_player_by_id(address)?;
            self.players.insert(player.get_id(), player.clone());
            Ok(player)
        }

        pub fn get_player_by_id(&self, id: &str) -> Result<&Arc<Player>, &'static str> {
            self.players.get(id).ok_or("Player not found.")
        }

        pub fn first_game_available(&mut self) -> Result<&mut Game, &'static str> {
            self.games.first_mut().ok_or("No games available.")
        }

        pub fn get_game_by_id(&mut self, id: &str) -> Result<&mut Game, &'static str> {
            self.games
                .iter_mut()
                .find(|game| game.id == id)
                .ok_or("Game not found.")
        }

        pub fn first_game_available_owned(&mut self) -> Result<Game, &'static str> {
            let first = self.games.pop().ok_or("No games available.")?;
            Ok(first)
        }

        pub fn get_scoreboards(&self) -> &[Scoreboard] {
            &self.scoreboards
        }

        pub fn get_scoreboard(&self, table_id: &str) -> Result<&Scoreboard, &'static str> {
            self.scoreboards
                .iter()
                .find(|scoreboard| scoreboard.id == table_id)
                .ok_or("Scoreboard not found searching by table_id")
        }

        pub fn drop_game(&mut self, id: &str) -> Result<Game, &'static str> {
            let (index, game) = self
                .games
                .iter()
                .enumerate()
                .find(|val| val.1.get_id() == id)
                .ok_or("Game not found.")?;

            if game.players.len() < 2 {
                Err("Minimum number of players not reached.")?;
            }

            let game = self.games.swap_remove(index);
            Ok(game)
        }

        pub fn generate_scoreboard_sync(&mut self, table: &Table) {
            let players = table.game.players.iter().cloned().collect();

            let winner = table.get_winner_sync();
            let scoreboard_id = table.id.clone();
            let hands = table.generate_hands();
            let scoreboard =
                Scoreboard::new(&scoreboard_id, table.game.get_id(), players, winner, hands);
            self.scoreboards.push(scoreboard);
        }

        pub async fn generate_scoreboard(&mut self, table: &Table) {
            let players = table.game.players.iter().cloned().collect();

            let winner = table.get_winner().await;
            let scoreboard_id = table.id.clone();
            let hands = table.generate_hands();
            let scoreboard =
                Scoreboard::new(&scoreboard_id, table.game.get_id(), players, winner, hands);
            self.scoreboards.push(scoreboard);
        }

        /**
         * Players are cleared from the game.
         */
        pub async fn reallocate_table_to_game(&mut self, table: Table) {
            self.generate_scoreboard(&table).await;

            let mut game = table.game;
            game.players.clear();
            self.add_game(game);
        }

        pub fn add_game(&mut self, game: Game) {
            self.games.push(game);
        }

        pub fn add_table(&mut self, table: Table) {
            self.tables.insert(table.get_id().to_owned(), table);
        }

        pub fn get_table(&self, id: &str) -> Option<&Table> {
            self.tables.get(id)
            // .or_else(|| self.tables.values().find(|table| table.game.get_id() == id))
        }

        pub fn get_table_mut(&mut self, id: &str) -> Result<&mut Table, &'static str> {
            if self.tables.is_empty() {
                return Err("No tables running.");
            }

            self.tables
                .get_mut(id)
                .ok_or("Table not found or not started.")
        }

        pub async fn stop_game(&mut self, table_id: &str) -> Result<(), &'static str> {
            info!("Stopping game table_id {}", table_id);

            let table = self
                .tables
                .remove(table_id)
                .ok_or("Table not found or not started.")?;

            self.reallocate_table_to_game(table).await;

            Ok(())
        }

        pub fn player_join(
            &mut self,
            game_id: &str,
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
        id: String,
        game_id: String,
        players: Vec<Arc<Player>>,
        winner: Option<Arc<Player>>,
        hands: Value,
    }
    impl Scoreboard {
        fn new(
            id: &str,
            game_id: &str,
            players: Vec<Arc<Player>>,
            winner: Option<Arc<Player>>,
            hands: Value,
        ) -> Self {
            info!(
                "Scoreboard {}; game_id {}; winner {:?}",
                id, game_id, winner
            );
            Scoreboard {
                id: id.to_string(),
                game_id: game_id.to_string(),
                players,
                winner,
                hands,
            }
        }

        pub fn to_json(&self) -> Value {
            let winner = self
                .winner
                .as_ref()
                .map_or("DRAW".to_string(), |player| player.name.clone());

            let value = json!({
                "id": self.id,
                "game_id": self.game_id,
                "players": self.players.iter().map(|player| player.name.clone()).collect::<Vec<_>>(),
                "winner": winner,
            });

            json!({
                "scoreboard": value,
                "hands": self.hands,
                "is_finished": true
            })
        }
    }

    /**
     * This is where the game is initialized.
     */
    pub struct Game {
        id: String,
        pub players: Vec<Arc<Player>>,
        manager: Option<Arc<Mutex<Manager>>>,
    }

    impl Default for Game {
        fn default() -> Self {
            Game {
                id: generate_id(),
                players: Vec::new(),
                manager: None,
            }
        }
    }

    impl Game {
        pub fn with_id(id: String) -> Self {
            Game {
                id,
                players: Vec::new(),
                manager: None,
            }
        }

        pub fn get_id(&self) -> &str {
            &self.id
        }

        pub fn new_with_manager_ref(manager: Arc<Mutex<Manager>>) -> Self {
            let mut game = Game::default();
            game.manager = Some(manager);
            game
        }

        // Transforms the game into a table.
        pub fn round_start(
            self,
            nth_decks: usize,
            last_timestamp: u64,
        ) -> Result<Table, &'static str> {
            if self.players.len() < 2 {
                Err("Minimum number of players not reached.")?;
            }

            Table::new(self, nth_decks, last_timestamp)
        }

        pub fn has_player(&self, id: &str) -> bool {
            self.players.iter().any(|player| player.get_id() == id)
        }
    }

    /**
     * The table is where the game is played.
     */
    pub struct Table {
        pub deck: Arc<Mutex<Deck>>,
        players_with_hand: Vec<PlayerHand>,
        game: Game,
        round: u8,
        id: String,
        // Cache for hand
        report: Option<Value>,
    }

    // TODO
    // impl Drop for Table {
    //     fn drop(&mut self) {
    //         let manager = self.game.manager.clone();

    //         if let Some(manager) = manager {
    //             let locker = manager.try_lock().map(|mut manager| {
    //                 manager.generate_scoreboard(self);

    //                 let mut game = Game::default();
    //                 game.manager = self.game.manager.clone();
    //                 manager.games.push(game);
    //             });

    //             if let Err(err) = locker {
    //                 error!("{}", err)
    //             }
    //         } else {
    //             error!("Table dont have reference")
    //         }
    //     }
    // }

    impl Table {
        fn new(game: Game, nth_decks: usize, last_timestamp: u64) -> Result<Self, &'static str> {
            // let bets = Vec::new();
            let players_with_hand = Vec::new();
            let deck = Deck::new_with_capacity(nth_decks).map(|deck| Arc::new(Mutex::new(deck)))?;

            let mut table = Self {
                deck,
                players_with_hand,
                game,
                round: 1,
                id: generate_id(),
                report: None,
            };

            table.game.players.iter().for_each(|player| {
                let player = player.clone();
                let player_hand = PlayerHand::new(player, table.deck.clone(), last_timestamp);
                table.players_with_hand.push(player_hand);
            });

            // @TODO: Implement bet.

            Ok(table)
        }

        pub fn get_round(&self) -> u8 {
            self.round
        }

        pub fn get_id(&self) -> &str {
            &self.id
        }

        pub fn get_name_player(&self, player_id: &str) -> Result<String, &'static str> {
            let player = self.get_player_by_id(player_id)?;
            Ok(player.get_player_ref().name.clone())
        }

        pub fn get_hand_size(&self) -> usize {
            self.players_with_hand.len()
        }

        pub fn get_points(&self, player_id: &str) -> Result<u8, &'static str> {
            let player = self.get_player_by_id(player_id)?;
            Ok(player.points)
        }

        fn get_hand_ref(&mut self, player_id: &str) -> Option<&mut PlayerHand> {
            self.players_with_hand
                .iter_mut()
                .find(|player| player_id == player.get_player_id())
        }

        pub fn is_any_player_has_condition(&self, condition: fn(&PlayerHand) -> bool) -> bool {
            self.players_with_hand
                .iter()
                .any(|player| condition(player))
        }

        pub fn is_all_players_has_condition(&self, condition: fn(&PlayerHand) -> bool) -> bool {
            self.players_with_hand
                .iter()
                .all(|player| condition(player))
        }

        pub async fn hit_player(
            &mut self,
            player_id: &str,
            timestamp: u64,
            seed: &str,
        ) -> Result<(), &'static str> {
            let table_round = self.round;
            let player = self.get_player_by_id_mut(player_id)?;
            let player_round = player.get_round();

            if table_round != player_round {
                info!(
                    "Game round {}; Player round {}; Player id {};",
                    table_round, player_round, player_id
                );
                Err("Round is not the same. Waiting for another players.")?;
            }

            player.hit(timestamp, seed).await?;

            self.next_round();

            self.regenerate_cache_hand();

            Ok(())
        }

        pub fn stand_player(
            &mut self,
            player_id: &str,
            last_timestamp: u64,
        ) -> Result<(), &'static str> {
            let player = self.get_player_by_id_mut(player_id)?;
            player.stand(last_timestamp)?;

            self.next_round();

            Ok(())
        }

        fn next_round(&mut self) {
            if self.can_advance_round() {
                self.round += 1;
            }
        }

        pub fn any_player_can_hit(&self) -> bool {
            self.players_with_hand
                .iter()
                .any(|player| !player.get_status_stand())
        }

        pub fn can_advance_round(&self) -> bool {
            info!("\nChecking if can advance round");
            let result = self.players_with_hand.iter().all(|player| {
                info!(
                    "Player {} round {}; Table round {} is_standing {} points {}",
                    player.get_name(),
                    player.get_round(),
                    self.round,
                    player.get_status_stand(),
                    player.points
                );

                player.get_status_stand() || self.round != player.get_round()
            });
            info!("Can advance {}\n", result);
            result
        }

        fn get_player_by_id_mut(&mut self, id: &str) -> Result<&mut PlayerHand, &'static str> {
            self.players_with_hand
                .iter_mut()
                .find(|player| player.get_player_id() == id)
                .ok_or("Player not found.")
        }

        #[cfg(test)]
        pub fn change_points(&mut self, player_id: &str, points: u8) -> Result<(), &'static str> {
            let hand = self.get_player_by_id_mut(player_id)?;
            hand.points = points;
            Ok(())
        }

        pub fn get_player_by_id(&self, id: &str) -> Result<&PlayerHand, &'static str> {
            self.players_with_hand
                .iter()
                .find(|player| player.get_player_id() == id)
                .ok_or("Player not found.")
        }

        pub fn regenerate_cache_hand(&mut self) {
            self.report = None;
            self.get_report_hand();
        }

        pub fn get_report_hand(&mut self) -> Value {
            if let Some(report) = &self.report {
                return report.clone();
            }

            let hands = self.generate_hands();
            let report = generate_report(hands);
            self.report = Some(report.clone());
            report
        }

        pub fn generate_hands(&self) -> Value {
            json!({
                "game_id": self.game.get_id(),
                "table_id": self.id,
                "players": self.players_with_hand.iter().map(|player| player.generate_hand()).collect::<Vec<_>>(),
                "is_finished": false,
                "round":self.round,
            })
        }

        pub(crate) async fn get_winner(&self) -> Option<Arc<Player>> {
            let mut winner: Option<Arc<Player>> = None;
            let mut winner_points = 0;

            // Safe for check hands, anyone cant pick a card.
            let _deck = self.deck.lock().await;

            for hand in self.players_with_hand.iter() {
                if (winner.is_none() || hand.points > winner_points) && hand.points <= 21 {
                    winner = Some(hand.get_player_ref());
                    winner_points = hand.points;
                } else if hand.points == winner_points {
                    winner = None;
                    break;
                }
            }

            winner
        }

        pub fn get_winner_sync(&self) -> Option<Arc<Player>> {
            let mut winner: Option<Arc<Player>> = None;
            let mut winner_points = 0;

            // Safe for check hands, anyone cant pick a card.
            let _deck = self.deck.try_lock().ok()?;

            for hand in self.players_with_hand.iter() {
                if (winner.is_none() || hand.points > winner_points) && hand.points <= 21 {
                    winner = Some(hand.get_player_ref());
                    winner_points = hand.points;
                } else if hand.points == winner_points {
                    winner = None;
                    break;
                }
            }

            winner
        }

        pub fn has_player(&self, player_id: &str) -> bool {
            let players = self
                .players_with_hand
                .iter()
                .map(|player| {
                    let id = player.get_player_id();
                    id
                })
                .collect::<Vec<_>>();

            let players = players.as_slice();

            info!("Searching for player {} in {:?}", player_id, players);

            self.players_with_hand
                .iter()
                .any(|player| player.get_player_id() == player_id)
        }

        pub fn get_players_len(&self) -> usize {
            self.players_with_hand.len()
        }
    }
}
