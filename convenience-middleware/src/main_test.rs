#[cfg(test)]
mod tests {
    // use super::*;
    use crate::{
        is_drand_beacon,
        models::models::Item,
        router::routes::{self},
    };
    use actix_web::{
        http::{self, header::ContentType},
        test,
    };
    use dotenv::dotenv;
    use serde_json::json;

    #[macro_export]
    macro_rules! check_if_dotenv_is_loaded {
        () => {{
            let is_env_loaded = dotenv().ok().is_some();
            assert!(is_env_loaded);
            is_env_loaded
        }};
    }

    #[actix_web::test]
    async fn test_index_ok() {
        // let req = test::TestRequest::default()
        //     .insert_header(ContentType::plaintext())
        //     .to_http_request();

        let resp = routes::hello().await;
        assert_eq!(resp.status(), http::StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_main_beacon() {
        check_if_dotenv_is_loaded!();

        let beacon = json!({
            "round": 3828300,
            "randomness": "7ff726d290836da706126ada89f7e99295c672d6768ec8e035fd3de5f3f35cd9",
            "signature": "ab85c071a4addb83589d0ecf5e2389f7054e4c34e0cbca65c11abc30761f29a0d338d0d307e6ebcb03d86f781bc202ee"
        });

        let payload = json!({
            "beacon": beacon,
        });

        let payload = payload.to_string();
        let payload = hex::encode(payload);
        let payload = format!("0x{}", payload);

        let object = json!({
            "data": {
                "payload": payload,
            }
        });

        let item = Item {
            request: object.to_string(),
        };

        let resp = is_drand_beacon(&item);
        assert_eq!(resp, true);
    }

    // #[actix_web::test]
    // async fn test_index_not_ok() {
    //     let req = test::TestRequest::default().to_http_request();
    //     let resp = routes::index(req).await;
    //     assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    // }
}