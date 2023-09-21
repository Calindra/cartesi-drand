use std::{env, mem::size_of, sync::Arc};

mod models;
mod rollups;
mod util;

use dotenv::dotenv;
use rollups::rollup::{handle_request_action, rollup, send_report};
use serde_json::Value;
use tokio::sync::{
    mpsc::{channel, Receiver, Sender},
    Mutex,
};

use crate::models::game::game::Manager;

fn start_handle_action(
    manager: Arc<Mutex<Manager>>,
    mut receiver: Receiver<Value>,
    sender_middleware: Sender<Value>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            let receive = receiver.recv().await;

            if let Some(value) = receive {
                println!("Received value: {}", value);

                let value = handle_request_action(&value, manager.clone(), true)
                    .await
                    .map_err(|err| {
                        eprintln!("Listener Error: {}", err);
                        err
                    });

                if let Ok(Some(report)) = value {
                    let _ = sender_middleware.send(report).await.map_err(|err| {
                        eprintln!("Send to middleware error: {}", err);
                        err
                    });
                }
            }
        }
    })
}

// Read from rollup and send to middleware
fn start_rollup(manager: Arc<Mutex<Manager>>, sender: Sender<Value>) {
    tokio::spawn(async move {
        loop {
            if let Err(resp) = rollup(manager.clone(), &sender).await {
                eprintln!("Sender error: {}", resp);
            }
        }
    });
}

// Send message to report
fn listener_send_message_to_middleware(mut receiver: Receiver<Value>) {
    tokio::spawn(async move {
        while let Some(value) = receiver.recv().await {
            println!("Send value to middleware: {}", value);
            let _ = send_report(value).await;
        }
    });
}

#[tokio::main]
async fn main() {
    const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");
    println!("BlackJack v{}", VERSION.unwrap_or("unknown"));

    dotenv().ok();
    env::var("MIDDLEWARE_HTTP_SERVER_URL").expect("Middleware http server must be set");

    const SLOTS: usize = 10;

    let manager = Arc::new(Mutex::new(Manager::new_with_games(SLOTS)));
    let (sender_rollup, receiver_rollup) = channel::<Value>(size_of::<Value>());
    let (sender_middl, receiver_middl) = channel::<Value>(size_of::<Value>());

    start_rollup(manager.clone(), sender_rollup); // 1
    listener_send_message_to_middleware(receiver_middl); // 3
    let _ = start_handle_action(manager, receiver_rollup, sender_middl).await; //2
}
