pub mod random {
    use std::{env::var as env, error::Error, ops::Range};

    use hyper::{body, client::HttpConnector, Body, Client, Request, Response, StatusCode};
    use rand::prelude::*;
    use rand_pcg::Pcg64;
    use rand_seeder::Seeder;
    use uuid::Uuid;

    pub fn generate_random_number(seed: String, range: Range<usize>) -> usize {
        let mut rng: Pcg64 = Seeder::from(seed).make_rng();
        rng.gen_range(range)
    }

    async fn call_random_api(
        client: &Client<HttpConnector>,
        request: Request<Body>,
    ) -> Result<(StatusCode, Response<Body>), Box<dyn Error>> {
        let response = client.request(request).await?;

        let status_response = response.status();
        println!("Receive random status {}", &status_response);

        Ok((status_response, response))
    }

    pub async fn need_seed(timestamp: u64) -> Result<String, Box<dyn Error>> {
        println!("Calling random...");

        let client = Client::new();
        let server_addr = env("MIDDLEWARE_HTTP_SERVER_URL")?;
        let uri = format!("{}/random?timestamp={}", server_addr, timestamp);

        loop {
            let request = Request::builder()
                .method("POST")
                .uri(uri.to_owned())
                .header("Content-Type", "application/json")
                .body(Body::empty())?;

            let (status_response, body) = call_random_api(&client, request).await?;

            match status_response {
                StatusCode::NOT_FOUND => {
                    println!("No pending random request, trying again");
                }

                StatusCode::OK => {
                    let body = body::to_bytes(body).await?;
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
}
