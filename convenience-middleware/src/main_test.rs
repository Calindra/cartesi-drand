#[cfg(test)]
mod middleware_tests {
    use std::sync::Once;

    use crate::{
        is_drand_beacon,
        models::models::{AppState, Beacon, Item},
        router::routes::{self}, utils::util::load_env_from_json, drand::get_drand_beacon,
    };
    use actix_web::{
        http::{self},
        test,
        web::{self},
        App,
    };
    use dotenv::dotenv;
    use http::Method;
    use httptest::{
        matchers::*,
        responders::{self, *},
        Expectation, ServerPool,
    };
    use serde_json::json;

    #[macro_export]
    macro_rules! check_if_dotenv_is_loaded {
        () => {{
            let is_env_loaded = dotenv().ok().is_some();
            assert!(is_env_loaded);
            load_env_from_json().await.unwrap();
            is_env_loaded
        }};
    }

    static SERVER: ServerPool = ServerPool::new(1);
    static BIND_SERVER: Once = Once::new();

    #[macro_export]
    macro_rules! mock_rollup_server {
        ($x:expr ) => {
            let server = SERVER.get_server();

            server.expect(
                Expectation::matching(request::method_path(
                    hyper::Method::POST.as_str(),
                    "/finish",
                ))
                .times(1..)
                .respond_with($x),
            );

            BIND_SERVER.call_once(|| {
                let url = server.url_str("");
                let url = url.trim_end_matches("/");
                std::env::set_var("ROLLUP_HTTP_SERVER_URL", url);
            });
        };
    }

    #[macro_export]
    macro_rules! call_random {
        ($app:expr) => {{
            let req = test::TestRequest::with_uri("/random?timestamp=1")
                .method(Method::GET)
                .to_request();
            let result = test::call_and_read_body($app, req).await;
            std::str::from_utf8(&result).unwrap().to_string()
        }};
    }
    #[macro_export]
    macro_rules! call_finish {
        ($app:expr) => {{
            let req = test::TestRequest::with_uri("/finish")
                .method(Method::POST)
                .set_json(json!({"status": "accept"}))
                .to_request();

            let result = test::call_and_read_body($app, req).await;
            let utf = std::str::from_utf8(&result).unwrap();
            let req: serde_json::Value = serde_json::from_str(utf).unwrap();
            req
        }};
    }

    #[actix_web::test]
    async fn test_is_drand_beacon() {
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
        mock_rollup_server!(status_code(202));

        let app_state = web::Data::new(AppState::new());
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
        assert!(
            resp.status().is_client_error(),
            "status: {:?}",
            status.as_str()
        );
        assert_eq!(resp.status(), 404);
        assert_eq!(
            manager.lock().await.pending_beacon_timestamp.get(),
            timestamp + app_state.safe_seconds
        );
    }

    #[actix_web::test]
    async fn request_random_with_new_beacon() {
        check_if_dotenv_is_loaded!();
        mock_rollup_server!(status_code(202));
        let last_clock_beacon = 24;

        let beacon = Beacon {
            round: 1,
            randomness: "to-be-a-seed".to_string(),
            timestamp: last_clock_beacon,
        };

        let app_state = web::Data::new(AppState::new());
        let manager = app_state.input_buffer_manager.clone();
        manager.lock().await.last_beacon.set(Some(beacon));

        let app = App::new()
            .app_data(app_state.clone())
            .service(routes::request_random);

        let app = test::init_service(app).await;

        let future_clock = last_clock_beacon + 10;

        let uri = format!("/random?timestamp={}", &future_clock);
        let req = test::TestRequest::with_uri(uri.as_str()).to_request();

        let resp = test::call_service(&app, req).await;
        let status = resp.status();
        assert!(status.is_client_error(), "status: {:?}", status.as_str());
        assert_eq!(status, 404);
        assert!(manager.lock().await.last_beacon.get_mut().is_some());
    }

    #[actix_web::test]
    async fn request_random_with_new_beacon_old_request() {
        check_if_dotenv_is_loaded!();

        let last_clock_beacon = 24;

        let beacon = Beacon {
            round: 1,
            randomness: "to-be-a-seed".to_string(),
            timestamp: last_clock_beacon,
        };

        let app_state = web::Data::new(AppState::new());
        let manager = app_state.input_buffer_manager.clone();
        manager.lock().await.last_beacon.set(Some(beacon));

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
        assert_eq!(status, 200);

        assert_eq!(manager.lock().await.flag_to_hold.is_holding, false);
        assert!(manager.lock().await.last_beacon.get_mut().is_some());
    }

    #[actix_web::test]
    async fn test_request_finish_without_input_to_respond() {
        check_if_dotenv_is_loaded!();
        mock_rollup_server!(status_code(202));

        let app_state = web::Data::new(AppState::new());

        let app = App::new()
            .app_data(app_state.clone())
            .service(routes::consume_buffer);

        let app = test::init_service(app).await;

        // the DApp call our middleware
        let req = test::TestRequest::with_uri("/finish")
            .method(Method::POST)
            .set_json(json!({"status": "accept"}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 202);
    }

    #[actix_web::test]
    async fn test_request_finish_with_input_to_respond() {
        check_if_dotenv_is_loaded!();
        mock_rollup_server!(json_encoded(
            json!({"data":{"metadata":{"block_number":241,"epoch_index":0,"input_index":0,"msg_sender":"0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266","timestamp":1689949250},"payload":"0x7B22696E707574223A2230783030227D"},"request_type":"advance_state"})
        ));

        let app_state = web::Data::new(AppState::new());

        let app = App::new()
            .app_data(app_state.clone())
            .service(routes::consume_buffer);

        let mut app = test::init_service(app).await;

        // the DApp call our middleware /finish
        let req = call_finish!(&mut app);
        assert_eq!(req["request_type"], "advance_state");
    }

    #[actix_web::test]
    async fn test_request_finish_with_beacon_inside_input() {
        check_if_dotenv_is_loaded!();
        mock_rollup_server!(responders::cycle![
            json_encoded(
                json!({"data":{"metadata":{"block_number":241,"epoch_index":0,"input_index":0,"msg_sender":"0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266","timestamp":1689949250},"payload":"0x7B22696E707574223A2230783030227D"},"request_type":"advance_state"})
            ),
            json_encoded(
                json!({"data":{"metadata":{"block_number":241,"epoch_index":0,"input_index":0,"msg_sender":"0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266","timestamp":1689949250},"payload":"0x7B22626561636F6E223A7B22726F756E64223A343038383031312C2272616E646F6D6E657373223A2239663032306331356262656539373437306532636562653566363030623636636363663630306236633031343931373535666661656638393365613733303039222C227369676E6174757265223A22623735613031613436386634396162646533623563383163303731336438313938343564313133626235613636626433613537366665343062313039323732373164396432356331633162626636366237336537623363326236333939363438227D2C22696E707574223A2230783030227D"},"request_type":"advance_state"})
            )
        ]);

        let app_state = web::Data::new(AppState::new());

        let app = App::new()
            .app_data(app_state.clone())
            .service(routes::consume_buffer)
            .service(routes::request_random);

        let mut app = test::init_service(app).await;

        // the DApp call our middleware to start something
        let req = call_finish!(&mut app);
        assert_eq!(req["request_type"], "advance_state");

        // call again and the beacon arrives
        let req = call_finish!(&mut app);
        assert_eq!(req["request_type"], "advance_state");

        // check randomness
        let randomness = call_random!(&mut app);
        assert_eq!(
            randomness,
            "29c0ecf5b324ed9710bddf053e5b4ec0f0faf002ccfcc9692214be6ef4110d29"
        );
    }

    #[actix_web::test]
    async fn test_request_finish_with_beacon_inside_input_scenario_2() {
        check_if_dotenv_is_loaded!();
        mock_rollup_server!(responders::cycle![
            json_encoded(
                json!({"data":{"metadata":{"block_number":241,"epoch_index":0,"input_index":0,"msg_sender":"0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266","timestamp":1689949250},"payload":"0x7B22696E707574223A2230783030227D"},"request_type":"advance_state"})
            ),
            json_encoded(
                json!({"data":{"metadata":{"block_number":241,"epoch_index":0,"input_index":0,"msg_sender":"0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266","timestamp":1689949250},"payload":"0x7b22626561636f6e223a7b22726f756e64223a343038383031312c2272616e646f6d6e657373223a2239663032306331356262656539373437306532636562653566363030623636636363663630306236633031343931373535666661656638393365613733303039222c227369676e6174757265223a22623735613031613436386634396162646533623563383163303731336438313938343564313133626235613636626433613537366665343062313039323732373164396432356331633162626636366237336537623363326236333939363438227d7d"},"request_type":"advance_state"})
            )
        ]);

        let app_state = web::Data::new(AppState::new());

        let app = App::new()
            .app_data(app_state.clone())
            .service(routes::consume_buffer)
            .service(routes::request_random);

        let mut app = test::init_service(app).await;

        // // the DApp call our middleware to start something
        let req = call_finish!(&mut app);
        assert_eq!(req["request_type"], "advance_state");

        // the DApp call our /random inside middleware to get a seed to generate a random number
        let randomness = call_random!(&mut app);

        // check randomness
        assert_eq!(
            randomness,
            "29c0ecf5b324ed9710bddf053e5b4ec0f0faf002ccfcc9692214be6ef4110d29"
        );
    }

    #[actix_web::test]
    async fn test_get_drand_beacon() {
        env_logger::init();
        check_if_dotenv_is_loaded!();
        let payload = "0x7b22626561636f6e223a7b22726f756e64223a343038383031312c2272616e646f6d6e657373223a2239663032306331356262656539373437306532636562653566363030623636636363663630306236633031343931373535666661656638393365613733303039222c227369676e6174757265223a22623735613031613436386634396162646533623563383163303731336438313938343564313133626235613636626433613537366665343062313039323732373164396432356331633162626636366237336537623363326236333939363438227d7d";
        let beacon = get_drand_beacon(payload);
        assert!(beacon.is_some());

        let payload = "0x7b22626561636f6e223a7b22726f756e64223a343038383031322c2272616e646f6d6e657373223a2239663032306331356262656539373437306532636562653566363030623636636363663630306236633031343931373535666661656638393365613733303039222c227369676e6174757265223a22623735613031613436386634396162646533623563383163303731336438313938343564313133626235613636626433613537366665343062313039323732373164396432356331633162626636366237336537623363326236333939363438227d7d";
        let beacon = get_drand_beacon(payload);
        assert!(beacon.is_none());

        let payload = "7b22626561636f6e223a7b22726f756e64223a343038383031312c2272616e646f6d6e657373223a2239663032306331356262656539373437306532636562653566363030623636636363663630306236633031343931373535666661656638393365613733303039222c227369676e6174757265223a2262373561303161343638663439616264653362356338316330373133643831393834356431313362623561363662643361353736666534306231303932373237316439643235633163316262663636623733653762336332623633393936343833333333227d7d";
        let beacon = get_drand_beacon(payload);
        assert!(beacon.is_none());
    }
}
