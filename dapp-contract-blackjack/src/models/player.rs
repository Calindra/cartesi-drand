pub mod player {
    use std::{
        error::Error,
        fmt::{self, Display},
        sync::Arc,
    };

    use serde::Serialize;
    use tokio::sync::Mutex;

    use crate::models::card::card::{Card, Deck, Rank};

    use crate::util::random::{call_seed, generate_random_number};

    pub struct Credit {
        pub amount: u32,
        pub symbol: String,
    }

    impl Display for Credit {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:} {:}", &self.amount, &self.symbol)
        }
    }

    pub struct Hand(pub Vec<Card>);

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
    pub struct Player {
        id: String,
        name: String,
    }

    impl Display for Player {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{{ id: {:}, name: {:} }}", &self.id, &self.name)
        }
    }

    impl Player {
        pub fn new(id: String, name: String) -> Self {
            Player { id, name }
        }

        pub fn new_without_id(name: String) -> Self {
            Player {
                id: bs58::encode(&name).into_string(),
                name,
            }
        }

        pub fn get_id(&self) -> String {
            self.id.to_owned()
        }
    }

    /**
     * Player's hand for specific round while playing.
     */
    pub struct PlayerHand {
        player: Arc<Player>,
        hand: Hand,
        pub points: u8,
        pub is_standing: bool,
        deck: Arc<Mutex<Deck>>,
        round: u8,
        // table: Table,
    }

    impl Display for PlayerHand {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
            let player_name = &self.player.name;
            write!(
                f,
                "{{ name: {:}, points: {:}, hand: {:} }}",
                player_name, &self.points, &self.hand
            )
        }
    }

    impl PlayerHand {
        pub fn new(player: Arc<Player>, deck: Arc<Mutex<Deck>>) -> Self {
            PlayerHand {
                player,
                hand: Hand(Vec::new()),
                is_standing: false,
                points: 0,
                deck,
                round: 1,
            }
        }

        pub fn generate_hand(&self) -> serde_json::Value {
            let hand = self
                .hand
                .0
                .iter()
                .map(|card| card.serialize())
                .collect::<Vec<String>>();

            serde_json::json!({
                "name": self.player.name,
                "points": self.points,
                "hand": hand,
            })
        }

        pub fn get_round(&self) -> u8 {
            self.round.clone()
        }

        pub fn get_player_id(&self) -> Result<String, Box<dyn Error>> {
            Ok(self.player.id.to_owned())
        }

        /**
         * Take a card from the deck and add it to the player's hand.
         */
        pub async fn hit(&mut self, timestamp: u64) -> Result<(), &'static str> {
            if self.points >= 21 {
                Err("Player is busted.")?;
            }

            if self.is_standing {
                Err("Already standing.")?;
            }

            let deck_is_empty = {
                let deck = self.deck.lock().await;
                deck.cards.is_empty()
            };

            if deck_is_empty {
                self.is_standing = true;
                Err("No cards in the deck.")?;
            }

            let seed = call_seed(timestamp).await.or(Err("No cant call seed"))?;

            let card = {
                let mut deck = self.deck.lock().await;
                let size = deck.cards.len();
                let nth = generate_random_number(seed, 0..size);
                let card = deck.cards.remove(nth);
                card
            };

            let card_point = card.show_point();
            let points = self.points + card_point;

            if card.rank == Rank::Ace && points > 21 {
                self.points = points - 10;
            } else {
                self.points = points;
            }

            self.is_standing = self.is_standing || self.points >= 21;
            self.hand.0.push(card);
            self.round += 1;

            Ok(())
        }

        /**
         * Add the value of the card to the player's hand.
         */
        pub async fn stand(&mut self) -> Result<(), &'static str> {
            self.is_standing = true;
            Ok(())
        }

        /**
         * Double the bet and take one more card.
         */
        async fn double_down(&mut self) -> Result<(), &'static str> {
            if self.is_standing {
                Err("Already standing.")?;
            }

            todo!();

            // let player = self.player.clone();

            // let player = player.lock().await;

            // let player_balance = player.balance.as_ref().ok_or("No balance.")?.amount;
            // let player_bet = player.bet.as_ref().ok_or("No bet.")?.amount;

            // let double_bet = player_bet.checked_mul(2).ok_or("Could not double bet.")?;

            // self.player.bet.as_mut().and_then(|credit| {
            //     credit.amount = double_bet;
            //     Some(credit)
            // });

            // self.hit().await?;
            // Ok(())
        }

        /**
         * Split the hand into two separate hands.
         */
        async fn split() {
            todo!();
        }

        /**
         * Give up the hand and lose half of the bet.
         */
        async fn surrender() {
            todo!();
        }
    }
}
