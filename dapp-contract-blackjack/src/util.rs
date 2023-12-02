pub struct Metadata {
    pub address: String,
    pub timestamp: u64,
}
pub mod random {
    use std::{env, error::Error, ops::Range, time::Duration};

    use log::{error, info};

    use hyper::{
        body::{self},
        Body, Client, Request, StatusCode,
    };
    use rand::prelude::*;
    use rand_pcg::Pcg64;
    use rand_seeder::Seeder;
    use tokio::time;
    use uuid::Uuid;

    pub fn generate_random_number(seed: &str, range: Range<usize>) -> usize {
        let mut rng: Pcg64 = Seeder::from(seed).make_rng();
        rng.gen_range(range)
    }

    pub async fn call_seed(timestamp: u64) -> Result<String, Box<dyn Error>> {
        let client = Client::new();

        let server_addr = env::var("MIDDLEWARE_HTTP_SERVER_URL")?;
        let server_addr = server_addr.trim_end_matches("/");

        let uri = format!("{}/random?timestamp={}", &server_addr, timestamp);

        info!("Calling random at {:}", &uri);

        loop {
            let request = Request::builder()
                .method(hyper::Method::GET)
                .uri(&uri)
                .header("Content-Type", "application/json")
                .body(Body::empty())?;

            let response = client.request(request).await?;

            let status_response = response.status();
            info!("Receive random status {}", &status_response);

            match status_response {
                StatusCode::BAD_REQUEST => {
                    let get_body = |response: hyper::Response<Body>| async {
                        let body_bytes = body::to_bytes(response.into_body()).await?;
                        let body_str = String::from_utf8(body_bytes.to_vec())?;
                        // let body_str = serde_json::to_string(&body_str)?;
                        Ok::<String, Box<dyn Error>>(body_str)
                    };

                    return match get_body(response).await {
                        Ok(body_str) => Err(format!(
                            "Bad request status code for random number with body: {body_str}",
                        )
                        .into()),
                        Err(error) => Err(format!(
                            "Bad request status code for random number with error: {}",
                            error.to_string()
                        )
                        .into()),
                    };
                }
                StatusCode::NOT_FOUND => {
                    return Err(
                        format!("No pending random request, trying again... uri = {uri}",).into(),
                    );
                    // info!("No pending random request, trying again... uri = {}", uri);
                    // time::sleep(Duration::from_secs(1)).await;
                }

                StatusCode::OK => {
                    let body = body::to_bytes(response).await?;
                    let body = String::from_utf8(body.to_vec())?;
                    return Ok(body);
                }

                code => {
                    // @todo doc this for production
                    // this is to avoid loop with inspect mode
                    return Err(format!("Unexpected status code {code} for random number").into());
                }
            }
        }
    }

    pub async fn retrieve_seed(timestamp: u64) -> Result<String, &'static str> {
        call_seed(timestamp).await.map_err(|error| {
            error!("Problem: {:}", error);
            "Cant get seed now"
        })
    }

    pub fn generate_id() -> String {
        Uuid::new_v4().to_string()
    }
}

pub mod json {
    use std::path::PathBuf;

    use log::info;
    use serde_json::{json, Value};
    use tokio::{
        fs::{read_to_string, File},
        io::{self, AsyncWriteExt},
    };

    use super::Metadata;

    pub fn decode_payload(payload: &str) -> Option<Value> {
        let payload = payload.trim_start_matches("0x");

        let payload = hex::decode(payload).ok()?;
        let payload = String::from_utf8(payload).ok()?;

        let payload = serde_json::from_str::<Value>(payload.as_str()).ok()?;

        Some(payload)
    }

    pub fn generate_report(payload: Value) -> Value {
        let payload = hex::encode(payload.to_string());
        let payload = format!("0x{}", payload);

        json!({
            "payload": payload,
        })
    }

    pub fn generate_message(payload: Value) -> Value {
        let payload = hex::encode(payload.to_string());
        let payload = format!("0x{}", payload);

        json!({
            "data": {
                "payload": payload,
            }
        })
    }

    pub async fn write_json(path: &PathBuf, obj: &Value) -> Result<(), io::Error> {
        let mut file = File::create(path).await?;
        let value = obj.to_string();
        file.write_all(value.as_bytes()).await?;
        Ok(())
    }

    pub async fn load_json(path: &PathBuf) -> Result<Value, io::Error> {
        info!("Trying read {:?}", path);

        let contents = read_to_string(path).await?;
        let value = serde_json::from_str::<Value>(&contents)?;
        Ok(value)
    }

    pub fn get_path_player(address_encoded: &str) -> PathBuf {
        let path = format!("./data/address/{}.json", address_encoded);
        PathBuf::from(&path)
    }

    pub fn get_path_player_name(name_encoded: &str) -> PathBuf {
        let path = format!("./data/names/{}.json", name_encoded);
        PathBuf::from(&path)
    }

    pub fn get_address_metadata_from_root(root: &Value) -> Option<Metadata> {
        let root = root.as_object()?;
        let root = root.get("data")?.as_object()?;
        let metadata = root.get("metadata")?.as_object()?;

        let address = metadata.get("msg_sender")?.as_str()?;
        let timestamp = metadata.get("timestamp")?.as_u64()?;
        // let input_index = metadata.get("input_index")?.as_u64()?;

        Some(Metadata {
            address: address.to_owned(),
            timestamp,
            // input_index,
        })
    }
}

pub mod pubkey {
    use std::{env, error::Error};

    use hyper::{Body, Client, Method, Request};
    use log::{error, info};

    #[derive(serde::Deserialize, serde::Serialize)]
    #[allow(non_snake_case)]
    pub struct DrandEnv {
        pub DRAND_PUBLIC_KEY: String,
        pub DRAND_PERIOD: Option<u64>,
        pub DRAND_GENESIS_TIME: Option<u64>,
        pub DRAND_SAFE_SECONDS: Option<u64>,
    }

    impl DrandEnv {
        pub fn new(
            pubkey: &str,
            period: Option<u64>,
            genesis_time: Option<u64>,
            safe_seconds: Option<u64>,
        ) -> Self {
            Self {
                DRAND_PUBLIC_KEY: pubkey.to_owned(),
                DRAND_PERIOD: period,
                DRAND_GENESIS_TIME: genesis_time,
                DRAND_SAFE_SECONDS: safe_seconds,
            }
        }
    }

    pub async fn call_update_key(drand_env: &DrandEnv) -> Result<(), Box<dyn Error>> {
        let body = serde_json::to_string(drand_env)?;

        let client = Client::new();
        let server_addr = env::var("MIDDLEWARE_HTTP_SERVER_URL")?;
        let server_addr = server_addr.trim_end_matches("/");

        let uri = format!("{}/update_drand_config", &server_addr);

        info!("Calling update key at {:}", &uri);

        let request = Request::builder()
            .method(Method::PUT)
            .uri(&uri)
            .header("Content-Type", "application/json")
            .body(Body::from(body))?;

        let response = client.request(request).await?;

        if response.status().is_success() {
            info!("Update key success");
            Ok(())
        } else {
            let msg = "Update key failed";
            error!("{}", msg);
            Err(msg.into())
        }
    }
}

#[cfg(test)]
pub mod env {
    #[allow(unused_macros)]
    macro_rules! check_if_dotenv_is_loaded {
        () => {{
            let is_env_loaded = dotenv::dotenv().ok().is_some();
            assert!(is_env_loaded);
            is_env_loaded
        }};
    }
    #[allow(unused_imports)]
    pub(crate) use check_if_dotenv_is_loaded;
}

pub mod logger {
    use log::{set_boxed_logger, Level, Log, Metadata, Record, SetLoggerError};

    pub struct SimpleLogger;

    impl Log for SimpleLogger {
        fn enabled(&self, metadata: &Metadata) -> bool {
            metadata.level() <= Level::Info
        }

        fn log(&self, record: &Record) {
            println!("DAPP CONTRACT {} - {}", record.level(), record.args());
        }

        fn flush(&self) {}
    }

    impl SimpleLogger {
        pub fn init() -> Result<(), SetLoggerError> {
            let logger = Box::new(SimpleLogger);
            set_boxed_logger(logger).map(|()| log::set_max_level(log::LevelFilter::Info))
        }
    }
}
