mod common;
#[path = "../src/main.rs"]
mod main;
#[path = "../src/models/mod.rs"]
mod models;
#[path = "../src/util.rs"]
mod util;

#[cfg(test)]
mod contract_blackjack_tests {
    use crate::{
        common::common::setup_hit_random,
        main::handle_request_action,
        models::{
            game::game::{Game, Manager},
            player::player::Player,
        },
        util::{env::check_if_dotenv_is_loaded, json::decode_payload},
    };

    use serde_json::json;
    use std::{borrow::BorrowMut, ops::Rem, sync::Arc};
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn should_create_manager_with_capacity() {
        let manager = Manager::new_with_games(10);
        assert_eq!(manager.games.len(), 10);
    }

    fn factory_message(payload: serde_json::Value) -> serde_json::Value {
        let payload = hex::encode(payload.to_string());
        let payload = format!("0x{}", payload);

        let metadata = json!({
            "msg_sender": "0xdeadbeef",
            "epoch_index": 0u64,
            "input_index": 0u64,
            "block_number": 123u64,
            "timestamp": 1690817064394u64,
        });

        json!({
            "data": {
                "metadata": metadata,
                "payload": payload,
            }
        })
    }

    #[tokio::test]
    async fn should_create_player() {
        let manager = Manager::default();
        let manager = Arc::new(Mutex::new(manager));

        // Based on this: https://docs.cartesi.io/cartesi-rollups/api/rollup/finish/
        let payload = json!({
            "input": {
                "name": "Bob",
                "action": "new_player"
            }
        });

        let data = factory_message(payload);

        let result = handle_request_action(&data, manager.clone(), false).await;

        assert!(result.is_ok(), "Result is not ok: {:}", result.unwrap_err());

        let manager = manager.lock().await;

        let size = manager.players.len();

        assert_eq!(size, 1);

        let player = manager.players.get(0).unwrap();

        println!("{:}", player);
    }

    #[tokio::test]
    async fn list_all_games_available() {
        // Generate manager with 10 games with empty players
        let mut manager = Manager::new_with_games(10);
        assert_eq!(manager.games.len(), 10);

        // Get ref for first game
        let game = manager.first_game_available().unwrap();
        let game_id = game.get_id().to_owned();

        // Add players
        for name in ["Alice", "Bob"] {
            let name = name.to_string();
            let player = Player::new_without_id(name);
            game.player_join(player).unwrap();
        }

        assert_eq!(game.players.len(), 2);

        // Start this game
        let game = manager.drop_game(game_id).unwrap();
        let table = game.round_start(1);

        assert!(table.is_ok(), "Table is not ok");

        // Generate ref to manager
        let manager = Arc::new(Mutex::new(manager));

        // Mock request from middleware
        let payload = json!({
            "input": {
                "action": "show_games"
            }
        });

        // Generate complete message with payload
        let data = factory_message(payload);

        // Call function used to see what action need for
        let response = handle_request_action(&data, manager.clone(), false).await;

        assert!(response.is_ok(), "Result is not ok");

        // Decode response for see if is ok
        let fn_response = || {
            let response = response.unwrap().unwrap();
            println!("{:}", &response);

            let response = response["data"]["payload"].as_str().unwrap();
            let response = decode_payload(response).unwrap();
            let response = response["games"].as_array().unwrap();
            response.to_owned()
        };

        let response = fn_response();

        assert_eq!(response.len(), 9);
    }

    #[tokio::test]
    async fn start_game() {
        let mut manager = Manager::new_with_games(10);
        let game = manager.first_game_available().unwrap();
        let game_id = game.get_id().to_owned();

        for name in ["Alice", "Bob"] {
            let name = name.to_string();
            let player = Player::new_without_id(name);
            game.player_join(player).unwrap();
        }

        let manager = Arc::new(Mutex::new(manager));

        // Mock request from middleware
        let payload = json!({
            "input": {
                "action": "start_game",
                "game_id": game_id,
            }
        });

        // Generate complete message with payload
        let data = factory_message(payload);

        // Call function used to see what action need for
        let response = handle_request_action(&data, manager.clone(), false).await;

        if let Err(err) = response {
            eprintln!("{:}", err);
        }

        assert!(response.is_ok());

        let manager = manager.lock().await;
        assert_eq!(manager.games.len(), 9);
        assert_eq!(manager.tables.len(), 1);
    }

    #[tokio::test]
    async fn only_player_inside_match_after_game_started() {
        let mut manager = Manager::new_with_games(10);
        let mut game = manager.first_game_available_owned().unwrap();

        for name in ["Alice", "Bob"] {
            let name = name.to_string();
            let player = Player::new_without_id(name);
            game.player_join(player).unwrap();
        }

        let table = game.round_start(1).unwrap();
        let size = table.players_with_hand.len();
        assert_eq!(size, 2);

        manager.realocate_table_to_game(table);
        let game = manager.first_game_available().unwrap();

        let player = Player::new_without_id("Eve".to_string());
        game.player_join(player).unwrap();

        assert_eq!(game.players.len(), 1);
    }

    #[tokio::test]
    async fn size_of_deck_when_game_started() {
        let mut game = Game::default();

        for name in ["Alice", "Bob"] {
            let player = Player::new_without_id(name.to_string());
            game.player_join(player).unwrap();
        }

        let table = game.round_start(1).unwrap();

        let size = table.deck.lock().await.cards.len();

        assert_eq!(size, 52);
    }

    #[tokio::test]
    async fn size_of_deck_while_players_hit() {
        check_if_dotenv_is_loaded!();
        let _server = setup_hit_random().await;

        let mut game = Game::default();

        for name in ["Alice", "Bob"] {
            let player = Player::new_without_id(name.to_string());
            game.player_join(player).unwrap();
        }

        let mut table = game.round_start(1).unwrap();
        let timestamp: u64 = 1691386341757;

        for player in table.players_with_hand.iter_mut() {
            let player = Box::new(player);

            while player.points <= 11 {
                let result = player.hit(timestamp).await;

                if let Err(err) = result {
                    eprintln!("{:}", err);
                }
            }
            println!("{:}", player);
        }

        let is_any_more_than_21 = table
            .players_with_hand
            .iter()
            .any(|player| player.points > 21);

        assert_eq!(is_any_more_than_21, false);
    }

    #[tokio::test]
    async fn hit_card_never_busted() {
        check_if_dotenv_is_loaded!();
        let _server = setup_hit_random().await;

        let mut manager = Manager::new_with_games(1);
        let mut game = manager.first_game_available_owned().unwrap();

        for name in ["Alice", "Bob"] {
            let player = Player::new_without_id(name.to_string());
            game.player_join(player).unwrap();
        }

        let mut table = game.round_start(1).unwrap();

        let first_player = table.players_with_hand.get_mut(0);

        assert!(first_player.is_some(), "First player not found.");

        let timestamp = 1691386341757;
        let first_player = first_player.unwrap();
        let mut i = 1;

        while first_player.points <= 11 {
            let res = first_player.hit(timestamp).await;

            assert!(res.is_ok(), "Player is busted.");

            let size = table.deck.lock().await.cards.len();

            assert_eq!(size.rem(52), (52 - i) % 52);

            println!("{:}", &first_player);
            i = i + 1;
        }
    }
}
