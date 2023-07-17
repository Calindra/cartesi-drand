mod main_test;
mod models;
mod router;
mod utils;

use crate::models::models::{AppState, Beacon, InputBufferManager, Item};
use crate::router::routes::{self};
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use drand_verify::{G2Pubkey, Pubkey};
use serde_json::{json, Value};
use std::{borrow::BorrowMut, env, sync::Arc};
use std::{error::Error, mem::size_of};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::{spawn, sync::Mutex};
use utils::util::deserialize_obj;

async fn rollup(
    sender: Sender<Item>,
    manager: Arc<Mutex<InputBufferManager>>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting rollup sender");

    let client = hyper::Client::new();
    let server_addr = env::var("ROLLUP_HTTP_SERVER_URL")?;

    let mut status = "accept";
    loop {
        println!("Sending finish");
        let response = json!({"status" : status.clone()});
        let request = hyper::Request::builder()
            .method(hyper::Method::POST)
            .header(hyper::header::CONTENT_TYPE, "application/json")
            .uri(format!("{}/finish", &server_addr))
            .body(hyper::Body::from(response.to_string()))?;
        let response = client.request(request).await?;
        println!("Received finish status {}", response.status());

        if response.status() == hyper::StatusCode::ACCEPTED {
            println!("No pending rollup request, trying again");
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
                    eprintln!("Unknown request type");
                    "reject"
                }
            };
        }
    }
}

async fn handle_inspect(
    client: &hyper::Client<hyper::client::HttpConnector>,
    server_addr: &str,
    request: Value,
    sender: &Sender<Item>,
    manager: &Arc<Mutex<InputBufferManager>>,
) -> Result<&'static str, Box<dyn std::error::Error>> {
    println!("req {:}", request);
    let payload = request["data"]["payload"]
        .as_str()
        .ok_or("Missing payload")?;
    let payload = payload.trim_start_matches("0x");
    let bytes: Vec<u8> = hex::decode(&payload).unwrap();
    let inspect_decoded = std::str::from_utf8(&bytes).unwrap();
    println!("Handling inspect {}", inspect_decoded);
    if inspect_decoded == "pending_drand_beacon" {
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
    println!("Handling advance");

    println!("req {:}", req);

    let _ = sender
        .send(Item {
            request: req.to_string(),
        })
        .await;

    Ok("accept")
}

fn start_senders(manager: Arc<Mutex<InputBufferManager>>, sender: Sender<Item>) {
    spawn(async move {
        let _ = rollup(sender, manager).await;
    });
}

/**
 * Example of a drand beacon request
 *
 * {"beacon":{"round":3828300,"randomness":"7ff726d290836da706126ada89f7e99295c672d6768ec8e035fd3de5f3f35cd9","signature":"ab85c071a4addb83589d0ecf5e2389f7054e4c34e0cbca65c11abc30761f29a0d338d0d307e6ebcb03d86f781bc202ee"}}
 */
fn is_drand_beacon(item: &Item) -> bool {
    let request = item.request.as_str();

    let json = match deserialize_obj(request) {
        Some(json) => json,
        None => return false,
    };

    if !json.contains_key("data") {
        return false;
    }

    let json = match json["data"].as_object() {
        Some(data) => data,
        None => return false,
    };

    if !json.contains_key("payload") {
        return false;
    }

    // @todo lidar com os unwraps
    let payload = json["payload"].as_str().unwrap();
    let payload = payload.trim_start_matches("0x");
    let payload = hex::decode(payload).unwrap();
    let payload = std::str::from_utf8(&payload).unwrap();

    let json = match deserialize_obj(payload) {
        Some(json) => json,
        None => return false,
    };

    let json = match json.get("beacon") {
        Some(beacon) => beacon,
        None => return false,
    };

    let json = match json.as_object() {
        Some(beacon) => beacon,
        None => return false,
    };

    if !json.contains_key("signature") || !json.contains_key("round") || !json["round"].is_number()
    {
        return false;
    }

    let key = env::var("PK_UNCHAINED_TESTNET").unwrap();
    let key = key.as_str();
    let mut pk = [0u8; 96];
    hex::decode_to_slice(key, pk.borrow_mut()).unwrap();

    let pk = match G2Pubkey::from_fixed(pk) {
        Ok(pk) => pk,
        Err(_) => return false,
    };

    let signature = json["signature"].as_str().unwrap();
    let signature = hex::decode(signature).unwrap();

    let round = json["round"].as_u64().unwrap();

    match pk.verify(round, b"", &signature) {
        Ok(check) => check,
        Err(_) => return false,
    }
}

fn start_listener(manager: Arc<Mutex<InputBufferManager>>, mut rx: Receiver<Item>) {
    spawn(async move {
        println!("Reading input from rollups receiver");
        let drand_period = env::var("DRAND_PERIOD").unwrap().parse::<u64>().unwrap();
        let drand_genesis_time = env::var("DRAND_GENESIS_TIME")
            .unwrap()
            .parse::<u64>()
            .unwrap();

        while let Some(item) = rx.recv().await {
            println!("Received item");
            println!("Request {}", item.request);

            let mut manager = match manager.try_lock() {
                Ok(manager) => manager,
                Err(_) => {
                    eprintln!("Failed to lock manager");
                    continue;
                }
            };

            if is_drand_beacon(&item) {
                println!("Received beacon");

                // Root Request
                let json = deserialize_obj(&item.request).unwrap();

                // Decript
                let json = json["data"]["payload"].as_str().unwrap();
                let json = json.trim_start_matches("0x");
                let json = hex::decode(json).unwrap();
                let json = std::str::from_utf8(&json).unwrap();

                // Root Payload
                let json = deserialize_obj(json).unwrap();
                let json = json["beacon"].as_object().unwrap();

                let round = json["round"].as_u64().unwrap();
                let beacon_time = (round * drand_period) + drand_genesis_time;

                manager.last_beacon.set(Some(Beacon {
                    timestamp: beacon_time,
                    metadata: json["randomness"].to_string(),
                }));
                manager.flag_to_hold.release();
                continue;
            } else {
                println!("Received a common input");
                // @todo devemos remover a nossa estrutura e deixar o input original?
                manager.messages.push_back(item);
                manager.request_count.set(manager.request_count.get() + 1);
            }
        }

        // manager.read_input_from_rollups(rx).await;
    });
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let (tx, rx) = channel::<Item>(size_of::<Item>());

    let app_state = web::Data::new(AppState {
        input_buffer_manager: InputBufferManager::new(),
    });

    let manager = app_state.input_buffer_manager.clone();
    start_senders(manager, tx);
    let manager = app_state.input_buffer_manager.clone();
    start_listener(manager, rx);

    println!("Starting server");

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(routes::index)
            .service(routes::request_random)
            .service(routes::consume_buffer)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
