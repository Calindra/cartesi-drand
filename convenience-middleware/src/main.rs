mod models;
mod routes;

use crate::{
    models::models::{AppState, Beacon, InputBufferManager, Item},
    routes::routes::{consume_buffer, index},
};
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use json::object;
use std::{error::Error, mem::size_of};
use tokio::spawn;
// use std::sync::mpsc::{channel, Receiver, Sender};
// use std::thread::spawn;
use std::{
    env,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc::{channel, Receiver, Sender};

async fn rollup(sender: Sender<Item>) -> Result<(), Box<dyn std::error::Error>> {
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
                "inspect_state" => handle_inspect(&client, &server_addr[..], req, &sender).await?,
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
        let report = object! {"payload" => format!("{}", "0x01")};
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

fn start_senders(sender: Sender<Item>) {
    spawn(async move {
        let _ = rollup(sender).await;
    });
}

fn is_drand_beacon(item: &Item) -> bool {
    let json = json::from(item.request.as_str());

    json.has_key("beacon")
}

fn start_listener(manager: Arc<Mutex<InputBufferManager>>, mut rx: Receiver<Item>) {
    spawn(async move {
        println!("Reading input from rollups receiver");

        while let Some(item) = rx.recv().await {
            println!("Received item");
            println!("Request {}", item.request);

            let mut manager = match manager.lock() {
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

    // let managerA = Arc::clone(&app_state.input_buffer_manager);
    start_senders(tx);
    let manager = Arc::clone(&app_state.input_buffer_manager);
    start_listener(manager, rx);

    println!("Starting server");

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(index)
            .service(consume_buffer)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
