use std::{env, mem::size_of, str::from_utf8, sync::Arc};

mod models;
mod rollups;
mod util;

use dotenv::dotenv;
use hyper::{body::to_bytes, header, Body, Client, Method, Request, StatusCode};
use log::{error, info, warn};
use rollups::rollup::{
    handle_advance, handle_inspect, handle_request_action, rollup, send_report, wait_func,
};
use serde_json::{from_str, json, Value};
use tokio::sync::{
    mpsc::{channel, Receiver, Sender},
    Mutex,
};

use crate::{models::game::game::Manager, util::logger::SimpleLogger};

fn start_handle_action(
    manager: Arc<Mutex<Manager>>,
    mut receiver: Receiver<Value>,
    sender_middleware: Sender<Value>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            let receive = receiver.recv().await;

            if let Some(value) = receive {
                info!("Received value: {}", value);

                let value = handle_request_action(&value, manager.clone(), true)
                    .await
                    .map_err(|err| {
                        error!("Listener Error: {}", err);
                        err
                    });

                if let Ok(Some(report)) = value {
                    let _ = sender_middleware.send(report).await.map_err(|err| {
                        error!("Send to middleware error: {}", err);
                        err
                    });
                }
            }
        }
    })
}

// Read from rollup and send to middleware
async fn start_rollup(manager: Arc<Mutex<Manager>>, sender: Sender<Value>) {
    // rollup(manager.clone(), &sender).await;
    tokio::spawn(async move {
        loop {
            if let Err(resp) = rollup(manager.clone(), &sender).await {
                error!("Sender error: {}", resp);
            }
        }
    });
}

// Send message to report
fn listener_send_message_to_middleware(mut receiver: Receiver<Value>) {
    tokio::spawn(async move {
        while let Some(value) = receiver.recv().await {
            info!("Send value to middleware: {}", value);
            let _ = send_report(value).await;
        }
    });
}

#[tokio::main]
async fn main() {
    SimpleLogger::init().expect("Logger error");

    const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");
    info!("BlackJack v{}", VERSION.unwrap_or("unknown"));

    dotenv().ok();
    env::var("MIDDLEWARE_HTTP_SERVER_URL").expect("Middleware http server must be set");

    const SLOTS: usize = 10;

    let manager = Arc::new(Mutex::new(Manager::new_with_games(SLOTS)));
    let (sender_rollup, receiver_rollup) = channel::<Value>(size_of::<Value>());

    info!("Starting loop...");

    let client = Client::new();
    // let https = HttpsConnector::new();
    // let client = Client::builder().build::<_, hyper::Body>(https);
    let server_addr = env::var("MIDDLEWARE_HTTP_SERVER_URL").unwrap();

    let mut status = "accept";
    loop {
        info!("Sending finish");
        let response = json!({ "status": status.clone() });
        let request = Request::builder()
            .method(Method::POST)
            .header(header::CONTENT_TYPE, "application/json")
            .uri(format!("{}/finish", &server_addr))
            .body(Body::from(response.to_string()))
            .unwrap();
        let request = client.request(request).await;
        let response = match request {
            Ok(resp) => resp,
            Err(_) => continue,
        };
        let status_response = response.status();
        info!("Receive finish status {}", &status_response);

        if status_response == StatusCode::ACCEPTED {
            warn!("No pending rollup request, trying again");
        } else {
            let body = to_bytes(response).await.unwrap();
            let body = from_utf8(&body).unwrap();
            let body = from_str::<Value>(body).unwrap();

            let request_type = body["request_type"]
                .as_str()
                .ok_or("request_type is not a string")
                .unwrap();

            status = match request_type {
                "advance_state" => {
                    handle_advance(manager.clone(), &server_addr[..], body, &sender_rollup)
                        .await
                        .unwrap()
                }
                "inspect_state" => {
                    handle_inspect(manager.clone(), &server_addr[..], body, &sender_rollup)
                        .await
                        .unwrap()
                }
                &_ => {
                    error!("Unknown request type");
                    "reject"
                }
            }
        }
        #[cfg(not(target_arch = "riscv64"))]
        wait_func().await;
    }

    // let (sender_middl, receiver_middl) = channel::<Value>(size_of::<Value>());

    // start_rollup(manager.clone(), sender_rollup).await; // 1
    // // rollup(manager.clone(), &sender_rollup).await;
    // info!("end rollups?");
    // listener_send_message_to_middleware(receiver_middl); // 3
    // let _ = start_handle_action(manager, receiver_rollup, sender_middl).await; //2
}
