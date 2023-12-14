#[cfg(test)]
mod middleware_tests {
    use hex_literal::hex;
    use std::{error::Error, sync::Once};

    use crate::{
        drand::get_drand_beacon,
        models::structs::{AppState, Beacon, DrandBeacon},
        rollup::input::{RollupInput, RollupInputDataMetadata},
        router::routes::{self},
        utils::util::{generate_payload_hex, load_env_from_json},
    };
    use actix_web::{
        http::{self},
        middleware::Logger,
        test,
        web::{self},
        App,
    };
    use dotenv::dotenv;
    use drand_verify::{G2Pubkey, G2PubkeyRfc, Pubkey as _};
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
    static BIND_LOGGER: Once = Once::new();

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

    fn generate_log() -> Logger {
        BIND_LOGGER.call_once(|| {
            let env = env_logger::Env::default().default_filter_or("info");
            env_logger::builder()
                .parse_env(env)
                .format_timestamp(None)
                .is_test(true)
                .try_init()
                .unwrap();
        });

        Logger::default()
    }

    // ({"data":{"metadata":{"block_number":241,"epoch_index":0,"input_index":0,"msg_sender":"0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266","timestamp":1689949250},"payload":payload_empty},"request_type":"advance_state"})
    fn mock_factory(
        payload: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, Box<dyn Error>> {
        // random address to msg_sender
        let addr = String::from("0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266");
        // random timestamp
        let timestamp = 1689949250;

        let metadata = RollupInputDataMetadata::builder()
            .with_block_number(241)
            .with_address_sender(addr)
            .with_timestamp(timestamp)
            .build();

        let mut data = RollupInput::builder()
            .with_metadata(metadata)
            .with_request_type("advance_state".into());

        if let Some(payload) = payload {
            let payload = generate_payload_hex(payload)?;
            data = data.with_payload(payload);
        }

        let data = data.build();

        let json = serde_json::to_value(data)?;
        println!("mock_factory: {:?}", json);
        Ok(json)
    }

    #[actix_web::test]
    async fn request_random_without_beacon() {
        check_if_dotenv_is_loaded!();
        mock_rollup_server!(status_code(202));

        let app_state = web::Data::new(AppState::new());
        let manager = app_state.input_buffer_manager.clone();

        let logger = generate_log();
        let app = App::new()
            .wrap(logger)
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
        assert_eq!(resp.status(), 400);
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

        let beacon = Beacon::builder()
            .with_round(1)
            .with_randomness("to-be-a-seed".to_string())
            .with_timestamp(last_clock_beacon)
            .build();

        let app_state = web::Data::new(AppState::new());
        let manager = app_state.input_buffer_manager.clone();
        manager.lock().await.last_beacon.set(Some(beacon));

        let logger = generate_log();

        let app = App::new()
            .wrap(logger)
            .app_data(app_state.clone())
            .service(routes::request_random);

        let app = test::init_service(app).await;

        let future_clock = last_clock_beacon + 10;

        let uri = format!("/random?timestamp={}", &future_clock);
        let req = test::TestRequest::with_uri(uri.as_str()).to_request();

        let resp = test::call_service(&app, req).await;
        let status = resp.status();
        assert!(status.is_client_error(), "status: {:?}", status.as_str());
        assert_eq!(status, 400);
        assert!(manager.lock().await.last_beacon.get_mut().is_some());
    }

    #[actix_web::test]
    async fn request_random_with_new_beacon_old_request() {
        check_if_dotenv_is_loaded!();

        let last_clock_beacon = 24;

        let beacon = Beacon::builder()
            .with_round(1)
            .with_randomness("to-be-a-seed".to_string())
            .with_timestamp(last_clock_beacon)
            .build();

        let app_state = web::Data::new(AppState::new());
        let manager = app_state.input_buffer_manager.clone();
        manager.lock().await.last_beacon.set(Some(beacon));

        let logger = generate_log();

        let app = App::new()
            .wrap(logger)
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

        assert!(!manager.lock().await.flag_to_hold.is_holding);
        assert!(manager.lock().await.last_beacon.get_mut().is_some());
    }

    #[actix_web::test]
    async fn test_request_finish_without_input_to_respond() {
        check_if_dotenv_is_loaded!();
        mock_rollup_server!(status_code(202));

        let app_state = web::Data::new(AppState::new());

        let logger = generate_log();
        let app = App::new()
            .wrap(logger)
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

        let payload = mock_factory(None).unwrap();

        mock_rollup_server!(json_encoded(payload));

        let app_state = web::Data::new(AppState::new());

        let logger = generate_log();
        let app = App::new()
            .wrap(logger)
            .app_data(app_state.clone())
            .service(routes::consume_buffer);

        let mut app = test::init_service(app).await;

        // the DApp call our middleware /finish
        let req = call_finish!(&mut app);
        assert_eq!(req["request_type"], "advance_state");
    }

    #[actix_web::test]
    async fn test_request_finish_with_beacon_inside_input() {
        let empty = mock_factory(None).unwrap();

        let randomness =
            String::from("7ade997ac926a8cada6835a4a16dfb2d31e639c7ac4ea4b508d5d3829496b527");
        let signature = String::from("8f4c029827e0c1d6f5db875c1927bc79cb15188e046de5ad627cb7d1efce87b1f3de99a045b770632333a41af3abf352");

        let beacon = DrandBeacon::builder()
            .with_randomness(randomness)
            .with_round(2832127)
            .with_signature(signature)
            .build()
            .wrap();

        let beacon = mock_factory(Some(beacon)).unwrap();

        check_if_dotenv_is_loaded!();
        mock_rollup_server!(responders::cycle![
            json_encoded(empty),
            json_encoded(beacon)
        ]);

        let logger = generate_log();
        let app_state = web::Data::new(AppState::new());

        let app = App::new()
            .wrap(logger)
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
            "a0e68303b27400e78fd3170af2a5387f9a8fe291545f8461cafafd90fb0e7357"
        );
    }

    #[actix_web::test]
    async fn test_request_finish_with_beacon_inside_input_scenario_2() {
        check_if_dotenv_is_loaded!();
        let empty = mock_factory(None).unwrap();
        let beacon = json!({"beacon":{"randomness":"7ade997ac926a8cada6835a4a16dfb2d31e639c7ac4ea4b508d5d3829496b527","round":2832127,"signature":"8f4c029827e0c1d6f5db875c1927bc79cb15188e046de5ad627cb7d1efce87b1f3de99a045b770632333a41af3abf352"}});
        let beacon = mock_factory(Some(beacon)).unwrap();

        mock_rollup_server!(responders::cycle![
            json_encoded(empty),
            json_encoded(beacon)
        ]);

        let app_state = web::Data::new(AppState::new());
        let logger = generate_log();

        let app = App::new()
            .wrap(logger)
            .app_data(app_state.clone())
            .service(routes::consume_buffer)
            .service(routes::request_random);

        let mut app = test::init_service(app).await;

        // the DApp call our middleware to start something
        let req = call_finish!(&mut app);
        assert_eq!(req["request_type"], "advance_state");

        // the DApp call our /random inside middleware to get a seed to generate a random number
        let randomness = call_random!(&mut app);

        // check randomness
        assert_eq!(
            randomness,
            "a0e68303b27400e78fd3170af2a5387f9a8fe291545f8461cafafd90fb0e7357"
        );
    }

    #[actix_web::test]
    async fn test_get_drand_beacon() {
        generate_log();
        check_if_dotenv_is_loaded!();
        let payload = generate_payload_hex(
            json!({"beacon":{"round":2797373,"randomness":"a8482088c159d7a5c9a54cf599c686febbef00d97d0c460dfee3cd80ff371dd9","signature":"857325b9d4d9e81b32ff98f0da6fc2c6af02b10207ef148d2d39b82b77159d766a9edf8a68a793135890a7dfa66166a7"}}),
        ).unwrap();
        let beacon = get_drand_beacon(&payload).ok();
        assert!(beacon.is_some());

        let payload = generate_payload_hex(
            json!({"beacon":{"round":4088012,"randomness":"9f020c15bbee97470e2cebe5f600b66cccf600b6c01491755ffaef893ea73009","signature":"b75a01a468f49abde3b5c81c0713d819845d113bb5a66bd3a576fe40b10927271d9d25c1c1bbf66b73e7b3c2b6399648"}}),
        ).unwrap();
        let beacon = get_drand_beacon(&payload).ok();
        assert!(beacon.is_none());

        let payload = generate_payload_hex(
            json!({"beacon":{"round":4088011,"randomness":"9f020c15bbee97470e2cebe5f600b66cccf600b6c01491755ffaef893ea73009","signature":"b75a01a468f49abde3b5c81c0713d819845d113bb5a66bd3a576fe40b10927271d9d25c1c1bbf66b73e7b3c2b63996483333"}}),
        ).unwrap();
        let beacon = get_drand_beacon(&payload).ok();
        assert!(beacon.is_none());
    }

    // #[actix_web::test]
    // async fn test_update_key() {
    //     env_logger::builder().is_test(true).try_init().unwrap();
    //     check_if_dotenv_is_loaded!();
    //     let logger = Logger::default();

    //     let app_state = web::Data::new(AppState::new());
    //     let app = App::new()
    //         .wrap(logger)
    //         .app_data(app_state.clone())
    //         .service(routes::update_drand_config);

    //     let drand_env = serde_json::to_string(&DrandEnv {
    //         DRAND_PUBLIC_KEY: "0x123".to_string(),
    //         DRAND_PERIOD: None,
    //         DRAND_GENESIS_TIME: None,
    //         DRAND_SAFE_SECONDS: Some(1000),
    //     })
    //     .unwrap();

    //     let req = test::TestRequest::default()
    //         .set_json(drand_env)
    //         .to_http_request();
    // }
    #[actix_web::test]
    async fn test_verify_fast() {
        const PK_HEX3: [u8; 96] = hex!("a0b862a7527fee3a731bcb59280ab6abd62d5c0b6ea03dc4ddf6612fdfc9d01f01c31542541771903475eb1ec6615f8d0df0b8b6dce385811d6dcf8cbefb8759e5e616a3dfd054c928940766d9a5b9db91e3b697e5d70a975181e007f87fca5e");
        let pk = G2Pubkey::from_fixed(PK_HEX3).unwrap();

        // https://api3.drand.sh/dbd506d6ef76e5f386f41c651dcb808c5bcbd75471cc4eafa3f4df7ad4e4c493/public/1
        let signature = hex::decode("9544ddce2fdbe8688d6f5b4f98eed5d63eee3902e7e162050ac0f45905a55657714880adabe3c3096b92767d886567d0").unwrap();
        let round: u64 = 1;
        let result = pk.verify(round, b"", &signature).unwrap();
        assert!(result);
    }

    #[actix_web::test]
    async fn test_verify_quick() {
        const PK_HEX2: [u8; 96] = hex!("83cf0f2896adee7eb8b5f01fcad3912212c437e0073e911fb90022d3e760183c8c4b450b6a0a6c3ac6a5776a2d1064510d1fec758c921cc22b0e17e63aaf4bcb5ed66304de9cf809bd274ca73bab4af5a6e9c76a4bc09e76eae8991ef5ece45a");
        let pk = G2PubkeyRfc::from_fixed(PK_HEX2).unwrap();

        // https://api3.drand.sh/dbd506d6ef76e5f386f41c651dcb808c5bcbd75471cc4eafa3f4df7ad4e4c493/public/1
        let signature = hex::decode("a1d1b86acd60adb8ed8dafbc8efdd5ebe3914c42de11a0a0636cf42d22a15a4a3d129f155732bd874c62bd153a2a65bd").unwrap();
        let round: u64 = 2798644;
        let result = pk.verify(round, b"", &signature).unwrap();
        assert!(result);
    }
}
