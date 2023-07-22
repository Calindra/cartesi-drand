pub mod routes {
    use std::env;

    use actix_web::{get, post, web, HttpResponse, Responder};
    use sha3::{Digest, Sha3_256};

    use crate::{
        models::models::{AppState, Beacon, RequestRollups, Timestamp},
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
            // consume from buffer
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
        if rollup_input.request_type == "advance_state" {
            if let Some(beacon) = get_drand_beacon(&rollup_input.data.payload) {
                println!("Is Drand!!! {:?}", beacon);
                let beacon_time = (beacon.round * ctx.drand_period) + ctx.drand_genesis_time;
                let manager = ctx.input_buffer_manager.try_lock();
                manager.unwrap().last_beacon.set(Some(Beacon {
                    timestamp: beacon_time,
                    metadata: beacon.randomness,
                }));

                // This is a beacon, so we omit it from the DApp in the endpoint /finish.
                return HttpResponse::Accepted().finish();
            }
        }
        let body = serde_json::to_string(&rollup_input).unwrap();
        return HttpResponse::Ok().body(body);
    }

    #[get("/random")]
    async fn request_random(
        ctx: web::Data<AppState>,
        query: web::Query<Timestamp>,
    ) -> impl Responder {
        let mut manager = match ctx.input_buffer_manager.try_lock() {
            Ok(manager) => manager,
            Err(_) => return HttpResponse::BadRequest().finish(),
        };

        // temos que pensar melhor o hold para identificar o request inicial e deixar ele passar
        // if manager.flag_to_hold.is_holding {
        //     return HttpResponse::NotFound().into();
        // } else {
        //     manager.flag_to_hold.hold_up();
        // }
        match manager.last_beacon.take() {
            Some(beacon) => {
                println!("beacon time {}", beacon.timestamp);
                // comparamos se o beacon é suficientemente velho pra devolver como resposta
                if query.timestamp < beacon.timestamp - 3 {
                    let salt = manager.randomness_salt.take() + 1;
                    manager.randomness_salt.set(salt);

                    let mut hasher = Sha3_256::new();
                    hasher.update([beacon.metadata.as_bytes(), &salt.to_le_bytes()].concat());
                    let randomness = hasher.finalize();
                    manager.flag_to_hold.release();
                    manager.last_beacon.set(Some(beacon));
                    let mut counter = ctx.process_counter.lock().await;
                    *counter += 1;
                    HttpResponse::Ok().body(hex::encode(randomness))
                } else {
                    manager.set_pending_beacon_timestamp(query.timestamp);
                    manager.last_beacon.set(Some(beacon));
                    HttpResponse::NotFound().finish()
                }
            }
            None => {
                manager.set_pending_beacon_timestamp(query.timestamp);
                HttpResponse::NotFound().finish()
            }
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
