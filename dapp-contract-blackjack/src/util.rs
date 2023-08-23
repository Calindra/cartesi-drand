pub struct Metadata {
    pub address: String,
    pub timestamp: u64,
    // input_index: u64,
}
pub mod random {
    use std::{env, error::Error, ops::Range, time::Duration};

    use hyper::{body, Body, Client, Request, StatusCode};
    use rand::prelude::*;
    use rand_pcg::Pcg64;
    use rand_seeder::Seeder;
    use uuid::Uuid;

    pub fn generate_random_number(seed: String, range: Range<usize>) -> usize {
        let mut rng: Pcg64 = Seeder::from(seed).make_rng();
        rng.gen_range(range)
    }

    pub async fn call_seed(timestamp: u64) -> Result<String, Box<dyn Error>> {
        let client = Client::new();
        let server_addr = env::var("MIDDLEWARE_HTTP_SERVER_URL")?;
        let server_addr = server_addr.trim_end_matches("/");

        let uri = format!("{}/random?timestamp={}", &server_addr, timestamp);

        println!("Calling random at {:}", &uri);

        loop {
            let request = Request::builder()
                .method(hyper::Method::GET)
                .uri(&uri)
                .header("Content-Type", "application/json")
                .body(Body::empty())?;

            let response = client.request(request).await?;

            let status_response = response.status();
            println!("Receive random status {}", &status_response);

            match status_response {
                StatusCode::NOT_FOUND => {
                    println!("No pending random request, trying again... uri = {}", uri);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }

                StatusCode::OK => {
                    let body = body::to_bytes(response).await?;
                    let body = String::from_utf8(body.to_vec())?;
                    return Ok(body);
                }

                code => {
                    println!("Unknown status code {:}", code);
                }
            }
        }
    }

    pub fn generate_id() -> String {
        Uuid::new_v4().to_string()
    }
}

pub mod json {
    use serde_json::{json, Value};
    use tokio::{fs::File, io::{AsyncWriteExt, self}};

    use super::Metadata;

    pub fn decode_payload(payload: &str) -> Option<Value> {
        let payload = payload.trim_start_matches("0x");

        let payload = hex::decode(payload).ok()?;
        let payload = String::from_utf8(payload).ok()?;

        let payload = serde_json::from_str::<Value>(payload.as_str()).ok()?;

        Some(payload)
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

    pub async fn write_json(path: &str, obj: &Value) -> Result<(), io::Error> {
        let mut file = File::create(path).await?;
        let value = obj.to_string();
        file.write_all(value.as_bytes()).await?;
        Ok(())
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

#[cfg(test)]
pub mod env {
    macro_rules! check_if_dotenv_is_loaded {
        () => {{
            let is_env_loaded = dotenv::dotenv().ok().is_some();
            assert!(is_env_loaded);
            is_env_loaded
        }};
    }

    pub(crate) use check_if_dotenv_is_loaded;
}
