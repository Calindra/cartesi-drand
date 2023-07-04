use actix_web::{get, post, web, App, HttpServer, Responder};
use dotenv::dotenv;
use json::object;
use serde::Deserialize;
use std::{env, sync::Mutex};

#[derive(Deserialize, Clone)]
struct Item {
    request: String,
}

#[derive(Deserialize)]
struct InputBufferManager {
    data: Mutex<Vec<Item>>,
    flagToHold: bool,
}

#[derive(Deserialize)]
struct AppState {
    input_buffer_manager: InputBufferManager,
}

impl InputBufferManager {
    fn new() -> InputBufferManager {
        InputBufferManager {
            data: Mutex::new(Vec::new()),
            flagToHold: false,
        }
    }

    fn read_input_from_rollups(&self) -> Result<(), String> {
        println!("Reading input from rollups");
        todo!("Implement this");
        // Ok(())
    }

    fn consume_input(&self) -> Option<Item> {
        println!("Consuming input");
        let mut buffer = self.data.lock().unwrap();
        let data = buffer.pop();
        data
    }

    fn await_beacon(&mut self) -> Result<(), String> {
        println!("Awaiting beacon");
        self.flagToHold = true;
        Ok(())
    }
}

#[get("/")]
async fn index() -> impl Responder {
    "Hello, World!"
}

#[post("/add")]
async fn add_to_buffer(item: web::Json<Item>, ctx: web::Data<AppState>) -> impl Responder {
    let mut buffer = ctx.input_buffer_manager.data.lock().unwrap();
    buffer.push(item.into_inner());
    let content = buffer
        .to_vec()
        .iter()
        .map(|x| x.request.clone())
        .collect::<Vec<String>>();
    format!("OK {}!", &content.join(","))
}

#[get("/consume")]
async fn consume_buffer(ctx: web::Data<AppState>) -> impl Responder {
    let input = ctx.input_buffer_manager.consume_input();

    let result = match input {
        Some(item) => item.request,
        None => "EMPTY".to_string(),
    };

    result
}

async fn rollup() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting rollup");

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
                "advance_state" => handle_advance(&client, &server_addr[..], req).await?,
                "inspect_state" => handle_inspect(&client, &server_addr[..], req).await?,
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
    req: json::JsonValue,
) -> Result<&'static str, Box<dyn std::error::Error>> {
    println!("Handling inspect");

    Ok("accept")
}

async fn handle_advance(
    client: &hyper::Client<hyper::client::HttpConnector>,
    server_addr: &str,
    req: json::JsonValue,
) -> Result<&'static str, Box<dyn std::error::Error>> {
    println!("Handling advance");

    Ok("accept")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let app_state = web::Data::new(AppState {
        input_buffer_manager: InputBufferManager::new(),
    });

    tokio::spawn(async move {
        rollup().await;
    });

    println!("Starting server");

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(index)
            .service(add_to_buffer)
            .service(consume_buffer)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
