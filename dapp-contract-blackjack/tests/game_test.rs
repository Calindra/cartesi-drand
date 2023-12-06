#[path = "../src/imports.rs"]
mod imports;
use imports::*;

#[cfg(test)]
mod game_tests {
    use std::sync::Arc;

    use crate::models::{game::prelude::Manager, player::player::Player};

    #[tokio::test]
    async fn get_winner_tests() {
        let mut manager = Manager::new_with_games(1);

        // Get ref for first game
        let game = manager.first_game_available().unwrap();
        let game_id = game.get_id().to_owned();

        // Add players
        let bob_address_owner = "f39Fd6e51aad88F6F4ce6aB8827279cffFb92266";
        let bob_address_encoded = bs58::encode(bob_address_owner).into_string();
        let bob = Player::new(bob_address_encoded.clone(), "Bob".to_owned());
        let bob = Arc::new(bob);
        manager.add_player(bob.clone()).unwrap();
        manager.player_join(&game_id, bob).unwrap();

        let alice_address = "70997970C51812dc3A010C7d01b50e0d17dc79C8";
        let alice_address_encoded = bs58::encode(alice_address).into_string();
        let alice = Player::new(alice_address_encoded.clone(), "Alice".to_owned());
        let alice = Arc::new(alice);
        manager.add_player(alice.clone()).unwrap();
        manager.player_join(&game_id, alice).unwrap();

        // Get game and make owner
        let game = manager.drop_game(&game_id).unwrap();

        // Generate table from game
        let timestamp: u64 = 1691386341757;
        let table = game.round_start(2, timestamp).unwrap();
        let table_id = table.get_id().to_owned();
        // Add table to manager
        manager.add_table(table);

        {
            let table = manager.get_table_mut(&table_id).unwrap();
            table.change_points(&bob_address_encoded, 20).unwrap();
            table.change_points(&alice_address_encoded, 21).unwrap();

            let winner = table.get_winner().await.unwrap();
            assert_eq!("Alice", winner.name);
        }
        {
            let table = manager.get_table_mut(&table_id).unwrap();
            table.change_points(&bob_address_encoded, 20).unwrap();
            table.change_points(&alice_address_encoded, 19).unwrap();
            let winner = table.get_winner().await.unwrap();
            assert_eq!("Bob", winner.name);
        }
        {
            let table = manager.get_table_mut(&table_id).unwrap();
            table.change_points(&bob_address_encoded, 20).unwrap();
            table.change_points(&alice_address_encoded, 20).unwrap();
            let winner = table.get_winner().await;
            assert!(winner.is_none());
        }
        {
            let table = manager.get_table_mut(&table_id).unwrap();
            table.change_points(&bob_address_encoded, 20).unwrap();
            table.change_points(&alice_address_encoded, 22).unwrap();
            let winner = table.get_winner().await.unwrap();
            assert_eq!("Bob", winner.name);
        }
        {
            let table = manager.get_table_mut(&table_id).unwrap();
            table.change_points(&bob_address_encoded, 22).unwrap();
            table.change_points(&alice_address_encoded, 22).unwrap();
            let winner = table.get_winner().await;
            assert!(winner.is_none());
        }
    }
}
