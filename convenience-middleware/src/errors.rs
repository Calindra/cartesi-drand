use actix_web::{error, http::header::ContentType, HttpResponse};
use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum VerificationError {
    #[display(fmt = "Error updating drand config: {}", cause)]
    InvalidDrandConfig { cause: String },
    // #[display(fmt = "Timeout")]
    // Timeout,
    // #[display(fmt = "Invalid signature")]
    // InvalidSignature,
}

impl serde::Serialize for VerificationError {
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

impl error::ResponseError for VerificationError {
    fn status_code(&self) -> hyper::StatusCode {
        match *self {
            VerificationError::InvalidDrandConfig { .. } => hyper::StatusCode::BAD_REQUEST,
            // VerificationError::Timeout => hyper::StatusCode::REQUEST_TIMEOUT,
            // VerificationError::InvalidSignature => hyper::StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(self)
    }
}
