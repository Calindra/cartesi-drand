mod models;
mod routes;

use crate::{
    models::models::{AppState, Beacon, InputBufferManager, Item},
    routes::routes::{consume_buffer, index},
};
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use drand_verify::{G2Pubkey, Pubkey};
use json::object;
use std::{borrow::BorrowMut, env, sync::Arc};
use std::{error::Error, mem::size_of};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::{spawn, sync::Mutex};

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
        let response = object! {"status" => status.clone()};
        let request = hyper::Request::builder()
            .method(hyper::Method::POST)
            .header(hyper::header::CONTENT_TYPE, "application/json")
            .uri(format!("{}/finish", &server_addr))
            .body(hyper::Body::from(response.dump()))?;
        let response = client.request(request).await?;
        println!("Received finish status {}", response.status());

        if response.status() == hyper::StatusCode::ACCEPTED {
            println!("No pending rollup request, trying again");
        } else {
            let body = hyper::body::to_bytes(response).await?;
            let utf = std::str::from_utf8(&body)?;
            let req = json::parse(utf)?;

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
    request: json::JsonValue,
    sender: &Sender<Item>,
    manager: &Arc<Mutex<InputBufferManager>>,
) -> Result<&'static str, Box<dyn std::error::Error>> {
    println!("req {:}", request);
    let payload = request["data"]["payload"]
        .as_str()
        .ok_or("Missing payload")?;
    let bytes: Vec<u8> = hex::decode(&payload[2..]).unwrap();
    let inspect_decoded = std::str::from_utf8(&bytes).unwrap();
    println!("Handling inspect {}", inspect_decoded);
    if inspect_decoded == "pending_drand_beacon" {
        // todo: aqui tem que ser o timestamp mais recente do request de beacon em hex
        // manager.pending_beacon_timestamp 64bits => 8 bytes
        let manager = manager.lock().await;
        let x = manager.pending_beacon_timestamp.get();
        let report = object! {"payload" => format!("{x:#x}")};
        let req = hyper::Request::builder()
            .method(hyper::Method::POST)
            .header(hyper::header::CONTENT_TYPE, "application/json")
            .uri(format!("{}/report", server_addr))
            .body(hyper::Body::from(report.dump()))?;
        let _ = client.request(req).await?;
    } else {
        let _ = sender
            .send(Item {
                request: request.dump(),
            })
            .await;
    }
    Ok("accept")
}

async fn handle_advance(
    _client: &hyper::Client<hyper::client::HttpConnector>,
    _server_addr: &str,
    req: json::JsonValue,
    sender: &Sender<Item>,
) -> Result<&'static str, Box<dyn Error>> {
    println!("Handling advance");

    println!("req {:}", req);

    let _ = sender
        .send(Item {
            request: req.dump(),
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
    let json = json::from(item.request.as_str());

    if !json.has_key("beacon")
        || !json.has_key("signature")
        || !json.has_key("round")
        || !json["round"].is_number()
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

    let signature: &str = json["signature"].as_str().unwrap();
    let signature = hex::decode(signature).unwrap();

    let round: u64 = json["round"].as_u64().unwrap();

    match pk.verify(round, b"", &signature) {
        Ok(check) => check,
        Err(_) => return false,
    }
}

fn start_listener(manager: Arc<Mutex<InputBufferManager>>, mut rx: Receiver<Item>) {
    // let pk = G2Pubkey::from(pk).unwrap();

    spawn(async move {
        println!("Reading input from rollups receiver");

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
                manager.last_beacon.set(Some(Beacon {
                    timestamp: 0,
                    metadata: item.request,
                }));
                manager.flag_to_hold.release();
                continue;
            }

            manager.messages.push_back(item);
            manager.request_count.set(manager.request_count.get() + 1);
        }

        // manager.read_input_from_rollups(rx).await;
    });
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let (tx, rx) = channel::<Item>(size_of::<Item>());

    let app_state = web::Data::new(AppState {
        input_buffer_manager: Arc::new(Mutex::new(InputBufferManager::new())),
    });

    let manager = Arc::clone(&app_state.input_buffer_manager);
    start_senders(manager, tx);
    let manager = Arc::clone(&app_state.input_buffer_manager);
    start_listener(manager, rx);

    println!("Starting server");

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(index)
            .service(routes::routes::request_random)
            .service(consume_buffer)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
