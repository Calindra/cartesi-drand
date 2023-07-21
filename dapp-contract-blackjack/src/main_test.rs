#[cfg(test)]
mod test {
    use std::{ops::Rem, sync::Arc};

    use tokio::sync::Mutex;

    use crate::{Game, PlayerBet};

    #[tokio::test]
    async fn only_player_inside_match_after_game_started() {
        let mut game = Game::default();

        for name in ["Alice", "Bob"] {
            let player = PlayerBet::new(name.to_string());
            game.player_join(player).unwrap();
        }

        let table = game.round_start(1).unwrap();

        // Add a new player after the game has started.
        let player = PlayerBet::new("Eve".to_string());
        game.player_join(player).unwrap();

        let size = table.players_with_hand.len();

        assert_eq!(game.players.len(), 3);
        assert_eq!(size, 2);
    }

    #[tokio::test]
    async fn size_of_deck_when_game_started() {
        let mut game = Game::default();

        for name in ["Alice", "Bob"] {
            let player = PlayerBet::new(name.to_string());
            game.player_join(player).unwrap();
        }

        let table = game.round_start(1).unwrap();

        let size = table.deck.lock().await.cards.len();

        assert_eq!(size, 52);
    }

    #[tokio::test]
    async fn size_of_deck_while_players_hit() {
        let mut game = Game::default();

        for name in ["Alice", "Bob"] {
            let player = PlayerBet::new(name.to_string());
            game.player_join(player).unwrap();
        }

        let table = game.round_start(1).unwrap();

        let size = table.deck.lock().await.cards.len();

        assert_eq!(size, 52);

        let i = Arc::new(Mutex::new(0));
        let mut tasks = Vec::new();

        for mut player in table.players_with_hand {
            let i = i.clone();
            let run = || async {
                let task = tokio::spawn(async move {
                    while player.points <= 11 {
                        if let Err(res) = player.hit().await {
                            println!("{:}", res);
                            break;
                        } else {
                            let mut i = i.lock().await;
                            *i = *i + 1;
                        }
                    }
                });
                task.await
            };

            tasks.push(run);
        }

        for run_task in tasks {
            run_task().await.unwrap();
        }

        assert_ne!(*i.lock().await, 52);
    }

    #[tokio::test]
    async fn hit_card_never_busted() {
        let mut game = Game::default();

        for name in ["Alice", "Bob"] {
            let player = PlayerBet::new(name.to_string());
            game.player_join(player).unwrap();
        }

        let mut table = game.round_start(1).unwrap();

        let first_player = table.players_with_hand.get_mut(0);

        assert!(first_player.is_some(), "First player not found.");

        let first_player = first_player.unwrap();
        let mut i = 1;

        while first_player.points <= 11 {
            let res = first_player.hit().await;

            assert!(res.is_ok(), "Player is busted.");

            let size = table.deck.lock().await.cards.len();

            assert_eq!(size.rem(52), (52 - i) % 52);

            println!("{:}", &first_player);
            i = i + 1;
        }
    }
}
