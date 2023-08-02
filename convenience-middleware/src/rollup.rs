use hyper::{Body, Response};
use serde::{Deserialize, Serialize};

pub mod server {
    use hyper::{Body, Response};
    use serde_json::{json, Value};

    use super::{RollupInput, parse_input_from_response};

    pub(crate) async fn send_finish(status: &str) -> Result<Response<Body>, hyper::Error> {
        let server_addr = std::env::var("ROLLUP_HTTP_SERVER_URL").unwrap();
        println!("Sending finish to {}", &server_addr);
        let client = hyper::Client::new();
        let response = json!({"status" : status.clone()});
        let request = hyper::Request::builder()
            .method(hyper::Method::POST)
            .header(hyper::header::CONTENT_TYPE, "application/json")
            .uri(format!("{}/finish", &server_addr))
            .body(hyper::Body::from(response.to_string()))
            .unwrap();
        let response = client.request(request).await;

        match &response {
            Ok(response) => {
                println!("Received finish status {} from RollupServer", response.status());
            }
            Err(error) => {
                eprintln!("Error {:?}", error);
            }
        };

        response
    }

    pub(crate) async fn send_finish_and_retrieve_input(status: &str) -> Option<RollupInput> {
        let response = send_finish(status).await.ok()?;
        if response.status() == hyper::StatusCode::ACCEPTED {
            return None
        }
        match parse_input_from_response(response).await {
            Ok(input) => return Some(input),
            Err(error) => {
                println!("Error {:?}", error);
                return None;
            }
        };
    }

    pub(crate) async fn send_report(report: Value) -> Result<&'static str, Box<dyn std::error::Error>> {
        let server_addr = std::env::var("ROLLUP_HTTP_SERVER_URL").unwrap();
        let client = hyper::Client::new();
        let req = hyper::Request::builder()
            .method(hyper::Method::POST)
            .header(hyper::header::CONTENT_TYPE, "application/json")
            .uri(format!("{}/report", server_addr))
            .body(hyper::Body::from(report.to_string()))
            .unwrap();
        let _ = client.request(req).await?;
        Ok("accept")
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct RollupInput {
    pub(crate) data: RollupInputData,
    pub(crate) request_type: String,
}

impl RollupInput {
    pub(crate) fn decoded_inspect(&self) -> String {
        let payload = self.data.payload.trim_start_matches("0x");
        let bytes: Vec<u8> = hex::decode(&payload).unwrap();
        let inspect_decoded = std::str::from_utf8(&bytes).unwrap();
        inspect_decoded.to_string()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct RollupInputData {
    pub(crate) payload: String,
    pub(crate) metadata: Option<RollupInputDataMetadata>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct RollupInputDataMetadata {
    pub(crate) block_number: u128,
    pub(crate) epoch_index: u128,
    pub(crate) input_index: u128,
    pub(crate) msg_sender: String,
    pub(crate) timestamp: u64,
}

pub(crate) async fn parse_input_from_response(
    response: Response<Body>,
) -> Result<RollupInput, serde_json::Error> {
    let body = hyper::body::to_bytes(response).await.unwrap();
    let utf = std::str::from_utf8(&body).unwrap();
    let result_deserialization = serde_json::from_str::<RollupInput>(utf);
    return result_deserialization;
}
