#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        is_drand_beacon,
        models::models::{AppState, Beacon, InputBufferManager, Item},
        router::routes::{self},
    };
    use actix_web::{
        http::{self},
        test, web, App,
    };
    use dotenv::dotenv;
    use serde_json::json;
    use tokio::sync::Mutex;

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

    #[actix_web::test]
    async fn request_random_without_beacon() {
        check_if_dotenv_is_loaded!();

        let manager = InputBufferManager::default();

        let app_state = web::Data::new(AppState {
            input_buffer_manager: Arc::new(Mutex::new(manager)),
        });

        let manager = app_state.input_buffer_manager.clone();

        let app = App::new()
            .app_data(app_state.clone())
            .service(routes::request_random);

        let app = test::init_service(app).await;

        let timestamp = 10;

        let uri = format!("/random?timestamp={}", &timestamp);
        let req = test::TestRequest::with_uri(uri.as_str()).to_request();

        let resp = test::call_service(&app, req).await;
        let status = resp.status();
        assert!(status.is_client_error(), "status: {:?}", status.as_str());
        assert_eq!(
            manager.lock().await.pending_beacon_timestamp.get(),
            timestamp
        );
    }

    #[actix_web::test]
    async fn request_random_with_new_beacon() {
        check_if_dotenv_is_loaded!();

        let last_clock_beacon = 24;

        let beacon = Beacon {
            metadata: json!({
                "message": "some info about beacon",
            })
            .to_string(),
            timestamp: last_clock_beacon,
        };

        let manager = InputBufferManager::default();

        manager.last_beacon.set(Some(beacon));

        let app_state = web::Data::new(AppState {
            input_buffer_manager: Arc::new(Mutex::new(manager)),
        });

        let manager = app_state.input_buffer_manager.clone();

        let app = App::new()
            .app_data(app_state.clone())
            .service(routes::request_random);

        let app = test::init_service(app).await;

        let old_clock = last_clock_beacon + 10;

        let uri = format!("/random?timestamp={}", &old_clock);
        let req = test::TestRequest::with_uri(uri.as_str()).to_request();

        let resp = test::call_service(&app, req).await;
        let status = resp.status();
        assert!(status.is_client_error(), "status: {:?}", status.as_str());
        assert!(manager.lock().await.last_beacon.get_mut().is_some());
    }

    #[actix_web::test]
    async fn request_random_with_old_beacon() {
        check_if_dotenv_is_loaded!();

        let last_clock_beacon = 24;

        let beacon = Beacon {
            metadata: json!({
                "message": "some info about beacon",
            })
            .to_string(),
            timestamp: last_clock_beacon,
        };

        let manager = InputBufferManager::default();

        manager.last_beacon.set(Some(beacon));

        let app_state = web::Data::new(AppState {
            input_buffer_manager: Arc::new(Mutex::new(manager)),
        });

        let manager = app_state.input_buffer_manager.clone();

        let app = App::new()
            .app_data(app_state.clone())
            .service(routes::request_random);

        let app = test::init_service(app).await;

        let old_clock = last_clock_beacon - 10;

        let uri = format!("/random?timestamp={}", &old_clock);
        let req = test::TestRequest::with_uri(uri.as_str()).to_request();

        let resp = test::call_service(&app, req).await;
        let status = resp.status();
        assert!(status.is_success(), "status: {:?}", status.as_str());

        // let body = resp.into_body();
        // let body = body.try_into_bytes();

        // println!("body: {:?}", body);

        assert_eq!(manager.lock().await.flag_to_hold.is_holding, false);
        assert!(manager.lock().await.last_beacon.get_mut().is_some());
    }

    // #[actix_web::test]
    // async fn test_index_not_ok() {
    //     let req = test::TestRequest::default().to_http_request();
    //     let resp = routes::index(req).await;
    //     assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    // }
}
