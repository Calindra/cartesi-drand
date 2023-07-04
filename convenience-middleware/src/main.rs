use actix_web::{get, post, web, App, HttpServer, Responder};
use serde::Deserialize;
use std::sync::Mutex;

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(AppState {
        input_buffer_manager: InputBufferManager::new(),
    });

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
