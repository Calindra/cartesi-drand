mod drand;
mod logger;
mod main_test;
mod models;
mod rollup;
mod router;
mod utils;

use crate::models::models::AppState;
use crate::router::routes;
use crate::utils::util::load_env_from_json;
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use log::{info, set_boxed_logger, set_max_level};
use logger::SimpleLogger;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let logger = SimpleLogger;
    set_boxed_logger(Box::new(logger))
        .map(|_| set_max_level(log::LevelFilter::Info))
        .unwrap();

    dotenv().ok();
    load_env_from_json().await.unwrap();

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
