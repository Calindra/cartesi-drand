pub mod player {
    use std::{
        fmt::{self, Display},
        sync::Arc,
    };

    use tokio::sync::Mutex;

    use crate::models::card::card::{Card, Deck, Rank};

    use crate::util::random::generate_random_seed;

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
        name: String,
        balance: Option<Credit>,
    }

    /**
     * Player's hand for specific round while playing.
     */
    pub struct PlayerHand {
        player: Arc<Mutex<PlayerBet>>,
        hand: Hand,
        pub points: u8,
        pub is_standing: bool,
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
    pub struct PlayerBet {
        player: Player,
        bet: Option<Credit>,
    }

    impl PlayerBet {
        pub fn new(name: String) -> PlayerBet {
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
        pub fn new(player: Arc<Mutex<PlayerBet>>, deck: Arc<Mutex<Deck>>) -> PlayerHand {
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
        pub async fn hit(&mut self) -> Result<(), &'static str> {
            if self.points >= 21 {
                return Err("Player is busted.");
            }

            if self.is_standing {
                return Err("Already standing.");
            }

            // let nth = random::<usize>();
            let nth = generate_random_seed("blackjack".to_string());

            let mut deck = self.deck.lock().await;

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

            self.is_standing = self.is_standing || self.points >= 21;
            self.hand.0.push(card);

            Ok(())
        }

        /**
         * Add the value of the card to the player's hand.
         */
        async fn stand(&mut self) -> Result<(), ()> {
            self.is_standing = true;
            Ok(())
        }

        /**
         * Double the bet and take one more card.
         */
        async fn double_down(&mut self) -> Result<(), &'static str> {
            if self.is_standing {
                return Err("Already standing.");
            }

            let player = self.player.clone();

            let player = player.lock().await;

            let player_balance = player.player.balance.as_ref().ok_or("No balance.")?.amount;
            let player_bet = player.bet.as_ref().ok_or("No bet.")?.amount;

            let double_bet = player_bet.checked_mul(2).ok_or("Could not double bet.")?;

            // self.player.bet.as_mut().and_then(|credit| {
            //     credit.amount = double_bet;
            //     Some(credit)
            // });

            self.hit().await?;
            Ok(())
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
