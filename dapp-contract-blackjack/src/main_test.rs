#[cfg(test)]
mod test {
    use std::borrow::BorrowMut;

    use crate::{Game, Player};

    #[tokio::test]
    async fn start_game() {
        let mut game = Game::new();

        for i in 0..2 {
            let player = Player::new(format!("Player {}", i));

            game.player_join(player);
        }

        let table = game.round_start();

        let players = table.get_players();
        let mut players = players.try_lock().unwrap();

        assert_eq!(players.len(), 2);

        let first_player = players[0].borrow_mut();
        let index = first_player.hit(&table.deck);

        assert!(index.is_some());
    }
}
