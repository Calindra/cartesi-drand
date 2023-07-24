pub mod card {
    use std::fmt::Display;

    #[derive(Clone)]
    pub enum Suit {
        Spades,   // Espadas
        Hearts,   // Copas
        Diamonds, // Ouros
        Clubs,    // Paus
    }

    impl Display for Suit {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let suit = match self {
                Suit::Spades => "Espadas",
                Suit::Hearts => "Copas",
                Suit::Diamonds => "Ouros",
                Suit::Clubs => "Paus",
            };

            write!(f, "{}", suit)
        }
    }

    #[derive(Clone, PartialEq)]
    pub enum Rank {
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

    impl Display for Rank {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let rank = self.clone() as u8;
            let rank_name: String = {
                if rank > 1 && rank < 11 {
                    rank.to_string()
                } else {
                    match self {
                        Rank::Ace => "Ás".to_string(),
                        Rank::Jack => "Valete".to_string(),
                        Rank::Queen => "Dama".to_string(),
                        Rank::King => "Rei".to_string(),
                        _ => "".to_string(),
                    }
                }
            };

            write!(f, "{}", rank_name)
        }
    }

    pub struct Card {
        pub suit: Suit,
        pub rank: Rank,
    }

    impl Card {
        pub fn show_point(&self) -> u8 {
            let mut point: u8 = self.rank.clone() as u8;

            if self.rank == Rank::Ace {
                point = 11;
            } else if self.rank == Rank::Jack || self.rank == Rank::Queen || self.rank == Rank::King
            {
                point = 10;
            }

            point
        }
    }

    impl Display for Card {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:} de {:}", &self.rank, &self.suit)
        }
    }

    pub struct Deck {
        pub cards: Vec<Card>,
    }

    impl Deck {
        pub fn new_with_capacity(nth: usize) -> Result<Self, &'static str> {
            if nth < 1 || nth > 8 {
                eprintln!("Invalid number of decks.");
                Err("Invalid number of decks.")?;
            }

            let mut decks = Deck::default();

            for _ in 1..nth {
                let deck = Deck::default().cards;
                decks.cards.extend(deck);
            }

            Ok(decks)
        }
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
}
