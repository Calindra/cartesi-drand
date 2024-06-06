use actix_web::{error, http::header::ContentType, HttpResponse};
use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum CheckerError {
    #[display(fmt = "Error updating drand config: {}", cause)]
    InvalidDrandConfig {
        cause: String,
    },

    #[display(fmt = "Already inspecting")]
    AlreadyInspecting,

    #[display(fmt = "Error sending finish request to rollup")]
    SendRollupAndRetrieveInputError,

    #[display(fmt = "Error sending finish request to rollup")]
    ByPassInspect,

    #[display(fmt = "Unknown request type")]
    UnknownRequestType,

    #[display(fmt = "Store input to consume later")]
    StoreInputByPass,

    #[display(fmt = "Error getting beacon signature")]
    SignatureErrorBeacon,

    #[display(fmt = "Error getting randomness")]
    RandomnessError,

    #[display(fmt = "Error storing input")]
    StoreInputError
}

impl serde::Serialize for CheckerError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let response = serde_json::json!({
            "error": self.to_string(),
        })
        .to_string();

        serializer.serialize_str(&response)
    }
}

impl error::ResponseError for CheckerError {
    fn status_code(&self) -> hyper::StatusCode {
        match *self {
            CheckerError::InvalidDrandConfig { .. } => hyper::StatusCode::BAD_REQUEST,
            CheckerError::AlreadyInspecting => hyper::StatusCode::BAD_REQUEST,
            CheckerError::SendRollupAndRetrieveInputError => hyper::StatusCode::BAD_REQUEST,
            CheckerError::ByPassInspect => hyper::StatusCode::BAD_REQUEST,
            CheckerError::UnknownRequestType => hyper::StatusCode::BAD_REQUEST,
            CheckerError::StoreInputByPass => hyper::StatusCode::BAD_REQUEST,
            CheckerError::SignatureErrorBeacon => hyper::StatusCode::BAD_REQUEST,
            CheckerError::RandomnessError => hyper::StatusCode::BAD_REQUEST,
            CheckerError::StoreInputError => hyper::StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(self)
    }
}
