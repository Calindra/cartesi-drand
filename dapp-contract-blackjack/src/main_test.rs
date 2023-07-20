#[cfg(test)]
mod test {
    use std::borrow::BorrowMut;

    use crate::{Game, Player};

    #[tokio::test]
    async fn start_game() {
        let mut game = Game::new();

        for i in 1..3 {
            let player = Player::new(format!("Player {}", i));

            game.player_join(player);
        }

        let table = game.round_start();

        let size = match table.deck.try_lock() {
            Ok(deck) => deck.cards.len(),
            Err(_) => 0,
        };

        assert_eq!(size, 52);

        let players = table.get_players();
        let mut players = players.try_lock().unwrap();

        assert_eq!(players.len(), 2);

        for i in 1..10 {
            let mut deck = table.deck.try_lock().unwrap();

            let first_player = players[0].borrow_mut();
            first_player.hit(&mut deck).unwrap();

            assert_eq!(deck.cards.len(), 52 - i);

            println!("{:}", &first_player);
        }

        // let first_card_value = first_player.hand;
    }
}
