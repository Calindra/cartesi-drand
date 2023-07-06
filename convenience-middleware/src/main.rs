use actix_web::{get, post, web, App, HttpServer, Responder};
use dotenv::dotenv;
use json::object;
use std::{error::Error, mem::size_of};
use tokio::spawn;
// use std::sync::mpsc::{channel, Receiver, Sender};
// use std::thread::spawn;
use std::{
    borrow::BorrowMut,
    cell::Cell,
    collections::VecDeque,
    env,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc::{channel, Receiver, Sender};

#[derive()]
struct Item {
    request: String,
}

struct Flag {
    is_holding: bool,
}

#[derive()]
struct InputBufferManager {
    messages: VecDeque<Item>,
    flag_to_hold: Flag,
    request_count: Cell<usize>,
}

#[derive()]
struct AppState {
    input_buffer_manager: Arc<Mutex<InputBufferManager>>,
}

impl Flag {
    fn new() -> Flag {
        Flag { is_holding: true }
    }

    fn hold_up(&mut self) {
        self.is_holding = true;
    }

    fn release(&mut self) {
        self.is_holding = false;
    }
}

impl InputBufferManager {
    fn new() -> InputBufferManager {
        InputBufferManager {
            messages: VecDeque::new(),
            flag_to_hold: Flag::new(),
            request_count: Cell::new(0),
        }
    }

    async fn read_input_from_rollups(&mut self, mut receiver: Receiver<Item>) {
        println!("Reading input from rollups receiver");

        while let Some(item) = receiver.recv().await {
            println!("Received item");
            println!("Request {}", item.request);

            self.messages.push_back(item);
            self.request_count.set(self.request_count.get() + 1);
        }
    }

    fn consume_input(&mut self) -> Option<Item> {
        println!("Consuming input");
        let buffer = self.messages.borrow_mut();
        let data = buffer.pop_front();
        self.request_count.set(self.request_count.get() - 1);
        data
    }

    fn await_beacon(&mut self) {
        println!("Awaiting beacon");
        self.flag_to_hold.hold_up();
    }
}

#[get("/")]
async fn index() -> impl Responder {
    "Hello, World!"
}

// #[post("/add")]
// async fn add_to_buffer(item: web::Json<Item>, ctx: web::Data<AppState>) -> impl Responder {
//     let mut buffer = ctx.input_buffer_manager.data.lock().unwrap();
//     buffer.push_back(item.into_inner());
//     let content = buffer
//         .to_vec()
//         .iter()
//         .map(|x| x.request.clone())
//         .collect::<Vec<String>>();
//     format!("OK {}!", &content.join(","))
// }

#[get("/consume")]
async fn consume_buffer(ctx: web::Data<AppState>) -> impl Responder {
    let input = ctx.input_buffer_manager.lock().unwrap().consume_input();

    let result = match input {
        Some(item) => item.request,
        None => "EMPTY".to_string(),
    };

    result
}

// todo add endpoint to hold on next inputs from Random Server

#[post("/hold")]
async fn hold_buffer(ctx: web::Data<AppState>) -> impl Responder {
    let mut manager = ctx.input_buffer_manager.lock().unwrap();

    if manager.flag_to_hold.is_holding {
        return "Holding already";
    }

    manager.await_beacon();

    "OK"
}

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
    _client: &hyper::Client<hyper::client::HttpConnector>,
    _server_addr: &str,
    req: json::JsonValue,
) -> Result<&'static str, Box<dyn std::error::Error>> {
    println!("Handling inspect");

    println!("req {:}", req);

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

fn start_workers(sender: Sender<Item>) {
    spawn(async move {
        let _ = rollup(sender).await;
    });
}

fn start_listener(manager: Arc<Mutex<InputBufferManager>>, mut rx: Receiver<Item>) {
    spawn(async move {
        let mut manager = manager.lock().unwrap();

        println!("Reading input from rollups receiver");

        loop {
            let conn = rx.try_recv();

            let item = match conn {
                Ok(conn) => conn,
                Err(_) => {
                    println!("No input from rollups receiver");
                    continue;
                }
            };

            println!("Received item");
            println!("Request {}", item.request);

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

    start_workers(tx);

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
