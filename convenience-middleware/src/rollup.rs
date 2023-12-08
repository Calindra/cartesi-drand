pub mod server {
    use hyper::{Body, Response};
    use log::info;
    use serde_json::{json, Value};
    use std::error::Error;

    use super::input::{parse_input_from_response, RollupInput};

    pub async fn send_finish(status: &str) -> Result<Response<Body>, Box<dyn Error>> {
        let server_str = std::env::var("ROLLUP_HTTP_SERVER_URL").expect("Env is not set");
        info!("Sending finish to {}", &server_str);
        let client = hyper::Client::new();
        let response = json!({"status" : status});
        let request = hyper::Request::builder()
            .method(hyper::Method::POST)
            .header(hyper::header::CONTENT_TYPE, "application/json")
            .uri(format!("{}/finish", &server_str))
            .body(hyper::Body::from(response.to_string()))?;

        let response = client.request(request).await?;

        info!(
            "Received finish status {} from RollupServer",
            response.status()
        );
        Ok(response)
    }

    pub async fn send_finish_and_retrieve_input(
        status: &str,
    ) -> Result<RollupInput, Box<dyn Error>> {
        let response = send_finish(status).await?;

        if response.status() == hyper::StatusCode::ACCEPTED {
            return Err("Skip".into());
        }

        let result = parse_input_from_response(response).await?;

        Ok(result)
    }

    pub async fn send_report(report: Value) -> Result<&'static str, Box<dyn std::error::Error>> {
        let server_addr =
            std::env::var("ROLLUP_HTTP_SERVER_URL").expect("ROLLUP_HTTP_SERVER_URL is not set");
        let client = hyper::Client::new();
        let req = hyper::Request::builder()
            .method(hyper::Method::POST)
            .header(hyper::header::CONTENT_TYPE, "application/json")
            .uri(format!("{}/report", server_addr))
            .body(hyper::Body::from(report.to_string()))?;

        let _ = client.request(req).await?;
        Ok("accept")
    }
}

pub mod input {
    use crate::{models::structs::Item, utils::util::deserialize_obj};
    use hyper::{Body, Response};
    use serde::{Deserialize, Serialize};
    use std::error::Error;

    #[derive(Serialize, Deserialize, Debug, Default)]
    pub struct RollupInput {
        pub data: RollupInputData,
        pub request_type: String,
    }

    impl TryFrom<Item> for RollupInput {
        type Error = serde_json::Error;

        fn try_from(item: Item) -> Result<Self, Self::Error> {
            serde_json::from_str(&item.request)
        }
    }

    impl RollupInput {
        pub fn decoded_inspect(&self) -> Result<String, Box<dyn Error>> {
            let payload = self.data.payload.trim_start_matches("0x");
            let bytes: Vec<u8> = hex::decode(payload)?;
            let inspect_decoded = std::str::from_utf8(&bytes)?;
            Ok(inspect_decoded.to_string())
        }
    }

    #[derive(Serialize, Deserialize, Debug, Default)]
    pub struct RollupInputData {
        pub payload: String,
        pub metadata: Option<RollupInputDataMetadata>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct RollupInputDataMetadata {
        pub block_number: u8,
        pub epoch_index: u8,
        pub input_index: u8,
        pub msg_sender: String,
        pub timestamp: u64,
    }

    pub async fn parse_input_from_response(
        response: Response<Body>,
    ) -> Result<RollupInput, Box<dyn Error>> {
        let body = hyper::body::to_bytes(response).await?;
        let utf = std::str::from_utf8(&body)?;
        let result_deserialization = serde_json::from_str::<RollupInput>(utf)?;
        Ok(result_deserialization)
    }

    pub fn has_input_inside_input(input: &RollupInput) -> bool {
        let json = input.data.payload.trim_start_matches("0x");
        let json = match hex::decode(json) {
            Ok(json) => json,
            Err(_) => return false,
        };
        let json = match std::str::from_utf8(&json) {
            Ok(json) => json,
            Err(_) => return false,
        };
        let value = deserialize_obj(json);
        let value = match value {
            Some(json) => json,
            None => return false,
        };
        value.contains_key("input")
    }
}
