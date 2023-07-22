use hyper::{Body, Response};
use serde::{Deserialize, Serialize};

pub mod server {
    use hyper::{Body, Response};
    use serde_json::json;

    pub(crate) async fn send_finish(status: &str) -> Response<Body> {
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
        let response = client.request(request).await.unwrap();
        println!(
            "Received finish status {} from RollupServer",
            response.status()
        );
        return response;
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct RollupInput {
    pub(crate) data: RollupInputData,
    pub(crate) request_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct RollupInputData {
    pub(crate) payload: String,
    pub(crate) metadata: RollupInputDataMetadata,
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
