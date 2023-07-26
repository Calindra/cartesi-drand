pub mod game {
    use crate::models::{
        card::card::Deck,
        player::player::{Credit, Player, PlayerBet, PlayerHand},
    };
    use std::sync::Arc;
    use tokio::sync::Mutex;

    pub struct Manager {
        pub games: Vec<Game>,
        pub players: Vec<Player>,
    }

    impl Default for Manager {
        fn default() -> Self {
            Manager {
                games: Vec::new(),
                players: Vec::new(),
            }
        }
    }

    impl Manager {
        pub fn add_player(&mut self, player: Player) -> Result<(), &'static str> {
            self.players.push(player);
            Ok(())
        }
    }

    /**
     * This is where the game is initialized.
     */
    pub struct Game {
        pub players: Vec<Arc<Mutex<PlayerBet>>>,
    }

    impl Default for Game {
        fn default() -> Self {
            Game {
                players: Vec::new(),
            }
        }
    }

    impl Game {
        pub fn player_join(&mut self, player: PlayerBet) -> Result<(), &'static str> {
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
        fn new(
            players: &Vec<Arc<Mutex<PlayerBet>>>,
            nth_decks: usize,
        ) -> Result<Table, &'static str> {
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
