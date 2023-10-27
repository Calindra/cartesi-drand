use std::{env, mem::size_of, str::from_utf8, sync::Arc};

mod models;
mod rollups;
mod util;

use dotenv::dotenv;
use log::{error, info};
use rollups::rollup::rollup;
use tokio::sync::Mutex;

use crate::{models::game::game::Manager, util::logger::SimpleLogger};

// Read from rollup and send to handle
async fn start_rollup(manager: Arc<Mutex<Manager>>) {
    loop {
        if let Err(resp) = rollup(manager.clone()).await {
            error!("Rollup: {resp}");
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    env::var("MIDDLEWARE_HTTP_SERVER_URL").expect("Middleware http server must be set");

    SimpleLogger::init().expect("Logger error");

    const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");
    info!("BlackJack v{}", VERSION.unwrap_or("unknown"));

    const SLOTS: usize = 10;

    let manager = Arc::new(Mutex::new(Manager::new_with_games(SLOTS)));
    start_rollup(manager).await;
}
