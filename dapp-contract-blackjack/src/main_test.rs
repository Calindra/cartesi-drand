#[cfg(test)]
mod test {
    use std::{borrow::BorrowMut, ops::Rem};

    use crate::{Game, PlayerBet};

    #[tokio::test]
    async fn start_game() {
        let mut game = Game::default();

        for name in ["Alice", "Bob"] {
            let player = PlayerBet::new(name.to_string());

            game.player_join(player).unwrap();
        }

        let table = game.round_start(1).unwrap();

        let player = PlayerBet::new("Eve".to_string());
        game.player_join(player).unwrap();

        let size = match table.deck.try_lock() {
            Ok(deck) => deck.cards.len(),
            Err(_) => 0,
        };

        assert_ne!(size, 0);
        assert_eq!(size.rem(52), 0);

        let mut players = table.players_with_hand;

        assert_eq!(players.len(), 2);

        let first_player = players[0].borrow_mut();
        let mut i = 1;
        while first_player.points < 17 {
            if let Err(err) = first_player.hit() {
                eprintln!("{:}", err);
                break;
            }

            let deck = table.deck.try_lock().unwrap();
            assert_eq!(deck.cards.len().rem(52), (52 - i) % 52);

            println!("{:}", &first_player);
            i = i + 1;
        }
    }
}
