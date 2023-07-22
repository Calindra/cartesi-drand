pub mod routes {
    use actix_web::{get, post, web, HttpResponse, Responder};
    use serde_json::json;
    use sha3::{Digest, Sha3_256};

    use crate::{
        models::models::{AppState, RequestRollups, Timestamp},
        rollup::{self},
        utils::util::get_drand_beacon,
    };

    pub async fn hello() -> HttpResponse {
        HttpResponse::Ok().body("Hello, World!")
    }

    #[get("/")]
    async fn index() -> impl Responder {
        hello().await
    }

    #[post("/finish")]
    async fn consume_buffer(
        ctx: web::Data<AppState>,
        body: web::Json<RequestRollups>,
    ) -> impl Responder {
        println!("Received finish request from DApp {:?}", body);
        {
            // the DApp consume from buffer first
            let manager = ctx.input_buffer_manager.try_lock();
            match manager {
                Ok(mut manager) => {
                    match manager.consume_input() {
                        Some(item) => return HttpResponse::Ok().body(item.request),
                        _ => {}
                    };
                }
                Err(_) => return HttpResponse::NotFound().finish(),
            };
        }

        let response = rollup::server::send_finish("accept").await;
        if response.status() == hyper::StatusCode::ACCEPTED {
            return HttpResponse::Accepted().finish();
        }
        let rollup_input = match rollup::parse_input_from_response(response).await {
            Ok(input) => input,
            Err(error) => {
                println!("Error {:?}", error);
                return HttpResponse::Accepted().finish();
            }
        };
        match rollup_input.request_type.as_str() {
            "advance_state" => {
                if let Some(beacon) = get_drand_beacon(&rollup_input.data.payload) {
                    println!("Is Drand!!! {:?}", beacon);
                    ctx.keep_newest_beacon(beacon);
                }
            }
            "inspect_state" => {
                let payload = rollup_input.data.payload.trim_start_matches("0x");
                let bytes: Vec<u8> = hex::decode(&payload).unwrap();
                let inspect_decoded = std::str::from_utf8(&bytes).unwrap();
                if inspect_decoded == "pending_drand_beacon" {
                    let manager = ctx.input_buffer_manager.lock().await;
                    let x = manager.pending_beacon_timestamp.get();
                    let report = json!({ "payload": format!("{x:#x}") });
                    let _ = rollup::server::send_report(report).await;

                    // This is a specific inspect, so we omit it from the DApp
                    return HttpResponse::Accepted().finish();
                }
            }
            &_ => {
                eprintln!("Unknown request type");
            }
        };

        // Dispatch the input to the DApp
        let body = serde_json::to_string(&rollup_input).unwrap();
        return HttpResponse::Ok().body(body);
    }

    #[get("/random")]
    async fn request_random(
        ctx: web::Data<AppState>,
        query: web::Query<Timestamp>,
    ) -> impl Responder {
        let randomness: Option<String> = ctx.get_randomness_for_timestamp(query.timestamp).await;
        if let Some(randomness) = randomness {
            // we already have the randomness to continue the process
            HttpResponse::Ok().body(randomness)
        } else {
            // call finish to halt and wait the beacon
            let response = rollup::server::send_finish("accept").await;
            if response.status() == hyper::StatusCode::ACCEPTED {
                // no input at all
                return HttpResponse::NotFound().finish();
            }
            let rollup_input = match rollup::parse_input_from_response(response).await {
                Ok(input) => input,
                Err(error) => {
                    println!("Error {:?}", error);
                    return HttpResponse::NotFound().finish();
                }
            };
            match rollup_input.request_type.as_str() {
                "advance_state" => {
                    if let Some(beacon) = get_drand_beacon(&rollup_input.data.payload) {
                        println!("Is Drand!!! {:?}", beacon);
                        let b = ctx.keep_newest_beacon(beacon);
                        if query.timestamp < b.timestamp - 3 {
                            let manager = match ctx.input_buffer_manager.try_lock() {
                                Ok(manager) => manager,
                                Err(_) => return HttpResponse::BadRequest().finish(),
                            };
                            let salt = manager.randomness_salt.take() + 1;
                            manager.randomness_salt.set(salt);

                            let mut hasher = Sha3_256::new();
                            hasher.update([b.randomness.as_bytes(), &salt.to_le_bytes()].concat());
                            let randomness = hasher.finalize();
                            return HttpResponse::Ok().body(hex::encode(randomness));
                        }
                    }
                    // @todo: send the input to the buffer
                }
                "inspect_state" => {
                    let payload = rollup_input.data.payload.trim_start_matches("0x");
                    let bytes: Vec<u8> = hex::decode(&payload).unwrap();
                    let inspect_decoded = std::str::from_utf8(&bytes).unwrap();
                    if inspect_decoded == "pending_drand_beacon" {
                        let manager = ctx.input_buffer_manager.lock().await;
                        let x = manager.pending_beacon_timestamp.get();
                        let report = json!({ "payload": format!("{x:#x}") });
                        let _ = rollup::server::send_report(report).await;

                        // This is a specific inspect, so we omit it from the DApp
                        return HttpResponse::NotFound().finish();
                    } else {
                        // @todo: send the input to the buffer
                    }
                }
                &_ => {
                    eprintln!("Unknown request type");
                }
            };
            HttpResponse::NotFound().finish()
        }
    }

    #[post("/hold")]
    async fn hold_buffer(ctx: web::Data<AppState>) -> impl Responder {
        let mut manager = match ctx.input_buffer_manager.try_lock() {
            Ok(manager) => manager,
            Err(_) => return HttpResponse::BadRequest().finish(),
        };

        if manager.flag_to_hold.is_holding {
            return HttpResponse::Accepted().body("Holding already");
        }

        manager.await_beacon();

        HttpResponse::Ok().body("Holding")
    }
}
