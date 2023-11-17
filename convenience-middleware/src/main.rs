mod drand;
mod main_test;
mod models;
mod rollup;
mod router;
mod utils;

use crate::models::models::{AppState, InputBufferManager, Item};
use crate::router::routes;
use crate::utils::util::load_env_from_json;
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use log::{error, info};
use serde_json::{json, Value};
use std::error::Error;
use std::{env, sync::Arc};
use tokio::sync::mpsc::Sender;
use tokio::{spawn, sync::Mutex};

// Rollup Sender - only work on loop mode
async fn rollup(
    sender: Sender<Item>,
    manager: Arc<Mutex<InputBufferManager>>,
) -> Result<(), Box<dyn Error>> {
    info!("Starting rollup sender");

    let client = hyper::Client::new();
    let server_addr = env::var("ROLLUP_HTTP_SERVER_URL")?;

    let mut status = "accept";
    loop {
        info!("Sending finish");
        let response = json!({ "status" : status });
        let request = hyper::Request::builder()
            .method(hyper::Method::POST)
            .header(hyper::header::CONTENT_TYPE, "application/json")
            .uri(format!("{}/finish", &server_addr))
            .body(hyper::Body::from(response.to_string()))?;
        let response = client.request(request).await?;
        info!("Received finish status {}", response.status());

        if response.status() == hyper::StatusCode::ACCEPTED {
            info!("No pending rollup request, trying again");
        } else {
            let body = hyper::body::to_bytes(response).await?;
            let utf = std::str::from_utf8(&body)?;
            let req: Value = serde_json::from_str(utf)?;

            let request_type = req["request_type"]
                .as_str()
                .ok_or("request_type is not a string")?;
            status = match request_type {
                "advance_state" => handle_advance(&client, &server_addr[..], req, &sender).await?,
                "inspect_state" => {
                    handle_inspect(&client, &server_addr[..], req, &sender, &manager).await?
                }
                &_ => {
                    error!("Unknown request type");
                    "reject"
                }
            };
        }
    }
}

// Handlers
async fn handle_inspect(
    client: &hyper::Client<hyper::client::HttpConnector>,
    server_addr: &str,
    request: Value,
    sender: &Sender<Item>,
    manager: &Arc<Mutex<InputBufferManager>>,
) -> Result<&'static str, Box<dyn Error>> {
    info!("req {:}", request);
    let payload = request["data"]["payload"]
        .as_str()
        .ok_or("Missing payload")?;
    let payload = payload.trim_start_matches("0x");
    let bytes: Vec<u8> = hex::decode(&payload).unwrap();
    let inspect_decoded = std::str::from_utf8(&bytes).unwrap();
    info!("Handling inspect {}", inspect_decoded);
    if inspect_decoded == "pendingdrandbeacon" {
        // todo: aqui tem que ser o timestamp mais recente do request de beacon em hex
        // manager.pending_beacon_timestamp 64bits => 8 bytes
        let manager = manager.lock().await;
        let x = manager.pending_beacon_timestamp.get();
        let report = json!({ "payload": format!("{x:#x}") });
        let req = hyper::Request::builder()
            .method(hyper::Method::POST)
            .header(hyper::header::CONTENT_TYPE, "application/json")
            .uri(format!("{}/report", server_addr))
            .body(hyper::Body::from(report.to_string()))?;
        let _ = client.request(req).await?;
    } else {
        let _ = sender
            .send(Item {
                request: request.to_string(),
            })
            .await;
    }
    Ok("accept")
}

async fn handle_advance(
    _client: &hyper::Client<hyper::client::HttpConnector>,
    _server_addr: &str,
    req: Value,
    sender: &Sender<Item>,
) -> Result<&'static str, Box<dyn Error>> {
    info!("Handling advance");

    info!("req {:}", req);

    let _ = sender
        .send(Item {
            request: req.to_string(),
        })
        .await;

    Ok("accept")
}

// Publisher - only work on loop mode
fn start_senders(manager: Arc<Mutex<InputBufferManager>>, sender: Sender<Item>) {
    spawn(async move {
        let _ = rollup(sender, manager).await;
    });
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    load_env_from_json().await.unwrap();

    env_logger::init();

    let app_state = web::Data::new(AppState::new());

    info!("Starting server");

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(routes::request_random)
            .service(routes::consume_buffer)
            .service(routes::update_drand_config)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
