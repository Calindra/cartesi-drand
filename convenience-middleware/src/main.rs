mod drand;
mod main_test;
mod models;
mod rollup;
mod router;
mod utils;
mod errors;

use crate::models::models::AppState;
use crate::router::routes;
use crate::utils::util::load_env_from_json;
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use log::info;

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
