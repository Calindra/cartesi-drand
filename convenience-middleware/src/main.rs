mod main_test;
mod models;
mod router;
mod utils;
mod rollup;
mod drand;

use crate::models::models::{AppState, Beacon, InputBufferManager, Item};
use crate::router::routes;
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
) -> Result<(), Box<dyn Error>> {
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
) -> Result<&'static str, Box<dyn Error>> {
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
    let key = env::var("PK_UNCHAINED_TESTNET").unwrap();

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

    let payload = || {
        let payload = json["payload"].as_str()?;
        let payload = payload.trim_start_matches("0x");
        let payload = hex::decode(payload).ok()?;
        let payload = match std::str::from_utf8(&payload) {
            Ok(payload) => payload.to_owned(),
            Err(_) => return None,
        };
        Some(payload)
    };

    let payload = match payload() {
        Some(payload) => payload,
        None => return false,
    };

    let json = match deserialize_obj(payload.as_str()) {
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

    let key = key.as_str();
    let mut pk = [0u8; 96];
    let is_decoded_err = hex::decode_to_slice(key, pk.borrow_mut()).is_err();

    if is_decoded_err {
        return false;
    }

    let pk = match G2Pubkey::from_fixed(pk) {
        Ok(pk) => pk,
        Err(_) => return false,
    };

    let signature = || {
        let signature = json["signature"].as_str()?;
        let signature = hex::decode(signature).ok()?;
        Some(signature)
    };

    let signature = match signature() {
        Some(signature) => signature,
        None => return false,
    };

    let round = json["round"].as_u64();

    let round = match round {
        Some(round) => round,
        None => return false,
    };

    let is_valid_key = pk.verify(round, b"", &signature).ok().is_some();

    is_valid_key
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

            let mut manager = manager.lock().await;

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
                    round,
                    timestamp: beacon_time,
                    randomness: json["randomness"].to_string(),
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

    let app_state = web::Data::new(AppState::new());

    let manager = app_state.input_buffer_manager.clone();
    start_senders(manager, tx);
    let manager = app_state.input_buffer_manager.clone();
    start_listener(manager, rx);

    println!("Starting server");

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(routes::request_random)
            .service(routes::consume_buffer)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
