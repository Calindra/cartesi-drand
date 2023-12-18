pub mod server {
    use hyper::{Body, Response};
    use log::info;
    use serde_json::{json, Value};
    use std::error::Error;

    use super::input::RollupInput;

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

        let result = RollupInput::try_from_async(response).await?;

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
    use crate::{
        models::structs::Item,
        utils::util::{deserialize_obj, generate_payload_hex},
    };
    use hyper::{Body, Response};
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use std::error::Error;

    #[derive(Default, Debug)]
    pub enum RollupState {
        Advance,
        Inspect,
        #[default]
        Unknown,
    }

    impl From<&str> for RollupState {
        fn from(s: &str) -> Self {
            match s {
                "advance_state" => RollupState::Advance,
                "inspect_state" => RollupState::Inspect,
                _ => RollupState::Unknown,
            }
        }
    }

    impl RollupState {
        pub fn as_str(&self) -> &str {
            match self {
                RollupState::Advance => "advance_state",
                RollupState::Inspect => "inspect_state",
                RollupState::Unknown => "unknown",
            }
        }
    }

    impl<'de> Deserialize<'de> for RollupState {
        fn deserialize<D>(deserializer: D) -> Result<RollupState, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            Ok(s.as_str().into())
        }
    }

    impl Serialize for RollupState {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_str(self.as_str())
        }
    }

    #[derive(Serialize, Deserialize, Debug, Default)]
    pub struct RollupInput {
        pub data: RollupInputData,
        pub request_type: RollupState,
    }

    #[derive(Default)]
    pub struct RollupInputBuilder(RollupInput);

    impl RollupInputBuilder {
        pub fn with_payload(mut self, payload: String) -> Self {
            self.0.data.payload = payload;
            self
        }

        pub fn with_metadata(mut self, metadata: RollupInputDataMetadata) -> Self {
            self.0.data.metadata = Some(metadata);
            self
        }

        pub fn with_request_type(mut self, request_type: RollupState) -> Self {
            self.0.request_type = request_type;
            self
        }

        pub fn build(self) -> RollupInput {
            RollupInput {
                data: self.0.data,
                request_type: self.0.request_type,
            }
        }
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

        pub fn builder() -> RollupInputBuilder {
            RollupInputBuilder::default()
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct RollupInputData {
        pub payload: String,
        pub metadata: Option<RollupInputDataMetadata>,
    }

    impl Default for RollupInputData {
        fn default() -> Self {
            let payload = generate_payload_hex(json!({"input":"0x00"})).unwrap();

            Self {
                payload,
                metadata: Default::default(),
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug, Default)]
    pub struct RollupInputDataMetadata {
        pub block_number: u128,
        pub epoch_index: u8,
        pub input_index: u8,
        pub msg_sender: String,
        pub timestamp: u64,
    }

    impl RollupInputDataMetadata {
        pub fn builder() -> RollupInputDataMetadataBuilder {
            RollupInputDataMetadataBuilder::default()
        }
    }

    #[derive(Default)]
    pub struct RollupInputDataMetadataBuilder(RollupInputDataMetadata);

    impl RollupInputDataMetadataBuilder {
        pub fn with_block_number(mut self, block_number: u128) -> Self {
            self.0.block_number = block_number;
            self
        }

        pub fn with_epoch_index(mut self, epoch_index: u8) -> Self {
            self.0.epoch_index = epoch_index;
            self
        }

        pub fn with_input_index(mut self, input_index: u8) -> Self {
            self.0.input_index = input_index;
            self
        }

        pub fn with_address_sender(mut self, msg_sender: String) -> Self {
            self.0.msg_sender = msg_sender;
            self
        }

        pub fn with_timestamp(mut self, timestamp: u64) -> Self {
            self.0.timestamp = timestamp;
            self
        }

        pub fn build(self) -> RollupInputDataMetadata {
            self.0
        }
    }

    impl RollupInput {
        pub async fn try_from_async(response: Response<Body>) -> Result<Self, Box<dyn Error>> {
            let body = hyper::body::to_bytes(response).await?;
            let utf = std::str::from_utf8(&body)?;
            let result_deserialization = serde_json::from_str::<RollupInput>(utf)?;
            Ok(result_deserialization)
        }
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
