#[cfg(test)]
mod test {
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
        let players = players.try_lock().unwrap();

        assert_eq!(players.len(), 2);
    }
}
