mod drand;
mod errors;
mod main_test;
mod models;
mod rollup;
mod router;
mod utils;

use crate::models::models::AppState;
use crate::router::routes;
use crate::utils::util::load_env_from_json;
use actix_web::{middleware::Logger, web, App, HttpServer};
use dotenv::dotenv;
use log::info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().unwrap();
    load_env_from_json().await.unwrap();

    let env = env_logger::Env::default().default_filter_or("info");
    env_logger::builder()
        .parse_env(env)
        .format_timestamp(None)
        .try_init()
        .unwrap();

    let app_state = web::Data::new(AppState::new());

    info!("Starting server");

    HttpServer::new(move || {
        let logger = Logger::default();

        App::new()
            .wrap(logger)
            .app_data(app_state.clone())
            .service(routes::request_random)
            .service(routes::consume_buffer)
            .service(routes::update_drand_config)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
