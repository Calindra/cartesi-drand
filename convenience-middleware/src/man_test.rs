#[cfg(test)]
mod tests {
    use super::*;

      use actix_web::{
        http::{self, header::ContentType},
        test,
    };

    #[actix_web::test]
    async fn test_index_ok() {
        let req = test::TestRequest::default()
            .insert_header(ContentType::plaintext())
            .to_http_request();
        let resp = index(req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
    }

    // #[actix_web::test]
    // async fn test_index_not_ok() {
    //     let req = test::TestRequest::default().to_http_request();
    //     let resp = index(req).await;
    //     assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    // }
}