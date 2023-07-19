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

        let mut table = game.round_start();

        assert_eq!(table.deck.cards.len(), 52);

        let players = table.get_players();
        let mut players = players.try_lock().unwrap();

        assert_eq!(players.len(), 2);

        let first_player = players[0].borrow_mut();
        first_player.hit(&mut table.deck).unwrap();

        assert_eq!(table.deck.cards.len(), 51);
        println!("Player 1: {:?}", &first_player);
    }
}
