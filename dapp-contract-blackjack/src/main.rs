use std::{env, str::from_utf8, sync::Arc};

mod models;
mod rollups;
mod util;

use dotenv::dotenv;
use hyper::{body::to_bytes, header, Body, Client, Method, Request, StatusCode};
use log::{error, info, warn};
use rollups::rollup::{
    handle_advance, handle_inspect, wait_func,
};
use serde_json::{from_str, json, Value};
use tokio::sync::Mutex;

use crate::{models::game::game::Manager, util::logger::SimpleLogger};

#[tokio::main]
async fn main() {
    SimpleLogger::init().expect("Logger error");

    const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");
    info!("BlackJack v{}", VERSION.unwrap_or("unknown"));

    dotenv().ok();
    env::var("MIDDLEWARE_HTTP_SERVER_URL").expect("Middleware http server must be set");

    const SLOTS: usize = 10;

    let manager = Arc::new(Mutex::new(Manager::new_with_games(SLOTS)));

    info!("Starting loop...");

    let client = Client::new();
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
                    let result = handle_advance(manager.clone(), &server_addr[..], body).await;
                    match result {
                        Ok(resp) => resp,
                        Err(_) => "reject",
                    }
                }
                "inspect_state" => handle_inspect(manager.clone(), &server_addr[..], body)
                    .await
                    .unwrap(),
                &_ => {
                    error!("Unknown request type");
                    "reject"
                }
            }
        }
        #[cfg(not(target_arch = "riscv64"))]
        wait_func().await;
    }
}
