#[cfg(test)]
mod middleware_tests {
    use hex_literal::hex;
    use std::sync::Once;

    use crate::{
        drand::get_drand_beacon,
        models::models::{AppState, Beacon},
        router::routes::{self},
        utils::util::load_env_from_json,
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

        assert_eq!(manager.lock().await.flag_to_hold.is_holding, false);
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
        mock_rollup_server!(json_encoded(
            json!({"data":{"metadata":{"block_number":241,"epoch_index":0,"input_index":0,"msg_sender":"0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266","timestamp":1689949250},"payload":"0x7B22696E707574223A2230783030227D"},"request_type":"advance_state"})
        ));

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
        check_if_dotenv_is_loaded!();
        mock_rollup_server!(responders::cycle![
            json_encoded(
                json!({"data":{"metadata":{"block_number":241,"epoch_index":0,"input_index":0,"msg_sender":"0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266","timestamp":1689949250},"payload":"0x7B22696E707574223A2230783030227D"},"request_type":"advance_state"})
            ),
            json_encoded(
                json!({"data":{"metadata":{"block_number":241,"epoch_index":0,"input_index":0,"msg_sender":"0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266","timestamp":1689949250},"payload":"0x7B22626561636F6E223A7B22726F756E64223A343038383031312C2272616E646F6D6E657373223A2239663032306331356262656539373437306532636562653566363030623636636363663630306236633031343931373535666661656638393365613733303039222C227369676E6174757265223A22623735613031613436386634396162646533623563383163303731336438313938343564313133626235613636626433613537366665343062313039323732373164396432356331633162626636366237336537623363326236333939363438227D2C22696E707574223A2230783030227D"},"request_type":"advance_state"})
            )
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
        let logger = generate_log();

        let app = App::new()
            .wrap(logger)
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
        generate_log();
        check_if_dotenv_is_loaded!();
        // let payload = "0x7b22626561636f6e223a7b22726f756e64223a343038383031312c2272616e646f6d6e657373223a2239663032306331356262656539373437306532636562653566363030623636636363663630306236633031343931373535666661656638393365613733303039222c227369676e6174757265223a22623735613031613436386634396162646533623563383163303731336438313938343564313133626235613636626433613537366665343062313039323732373164396432356331633162626636366237336537623363326236333939363438227d7d";
        let payload = "0x7b22626561636f6e223a7b22726f756e64223a323739373337332c2272616e646f6d6e657373223a2261383438323038386331353964376135633961353463663539396336383666656262656630306439376430633436306466656533636438306666333731646439222c227369676e6174757265223a22383537333235623964346439653831623332666639386630646136666332633661663032623130323037656631343864326433396238326237373135396437363661396564663861363861373933313335383930613764666136363136366137227d7d";
        let beacon = get_drand_beacon(payload);
        assert!(beacon.is_some());

        let payload = "0x7b22626561636f6e223a7b22726f756e64223a343038383031322c2272616e646f6d6e657373223a2239663032306331356262656539373437306532636562653566363030623636636363663630306236633031343931373535666661656638393365613733303039222c227369676e6174757265223a22623735613031613436386634396162646533623563383163303731336438313938343564313133626235613636626433613537366665343062313039323732373164396432356331633162626636366237336537623363326236333939363438227d7d";
        let beacon = get_drand_beacon(payload);
        assert!(beacon.is_none());

        let payload = "7b22626561636f6e223a7b22726f756e64223a343038383031312c2272616e646f6d6e657373223a2239663032306331356262656539373437306532636562653566363030623636636363663630306236633031343931373535666661656638393365613733303039222c227369676e6174757265223a2262373561303161343638663439616264653362356338316330373133643831393834356431313362623561363662643361353736666534306231303932373237316439643235633163316262663636623733653762336332623633393936343833333333227d7d";
        let beacon = get_drand_beacon(payload);
        assert!(beacon.is_none());
    }

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
