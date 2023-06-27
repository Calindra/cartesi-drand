use actix_web::{get, post, web, App, HttpServer, Responder};
use serde::Deserialize;
use std::sync::Mutex;

#[derive(Deserialize)]
struct AppState {
    buffer: Mutex<Vec<String>>,
}

#[derive(Deserialize)]
struct Item {
    request: String,
}

#[get("/")]
async fn index() -> impl Responder {
    "Hello, World!"
}

#[get("/{name}")]
async fn hello(name: web::Path<String>) -> impl Responder {
    format!("Hello {}!", &name)
}

#[post("/add")]
async fn add_to_buffer(item: web::Json<Item>, ctx: web::Data<AppState>) -> impl Responder {
    let mut buffer = ctx.buffer.lock().unwrap();
    buffer.push(item.request.clone());
    let buffer = buffer.clone();
    format!("Hello {}!", &buffer.join(", "))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(AppState {
        buffer: Mutex::new(Vec::new()),
    });


    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(index)
            .service(hello)
            .service(add_to_buffer)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
