pub mod routes {
    use actix_web::{get, post, web, HttpResponse, Responder};

    use crate::{
        drand::{get_drand_beacon, is_querying_pending_beacon, send_pending_beacon_report},
        models::models::{AppState, RequestRollups, Timestamp},
        rollup::{server::send_finish_and_retrieve_input, has_input_inside_input},
    };

    #[post("/finish")]
    async fn consume_buffer(
        ctx: web::Data<AppState>,
        body: web::Json<RequestRollups>,
    ) -> impl Responder {
        println!("Received finish request from DApp {:?} version={}", body, ctx.version);

        // the DApp consume from the buffer first
        if let Some(item) = ctx.consume_input().await {
            if has_input_inside_input(&item.request) {
                return HttpResponse::Ok().body(item.request);
            } else {
                return HttpResponse::Accepted().finish();
            }
        }
        let rollup_input = match send_finish_and_retrieve_input("accept").await {
            Some(input) => input,
            None => return HttpResponse::Accepted().finish(),
        };
        match rollup_input.request_type.as_str() {
            "advance_state" => {
                if let Some(beacon) = get_drand_beacon(&rollup_input.data.payload) {
                    println!("Is Drand!!! {:?}", beacon);
                    ctx.keep_newest_beacon(beacon);
                }
            }
            "inspect_state" => {
                if is_querying_pending_beacon(&rollup_input) {
                    send_pending_beacon_report(&ctx).await;

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
        if has_input_inside_input(&body) {
            return HttpResponse::Ok().body(body);
        } else {
            return HttpResponse::Accepted().finish();
        }
    }

    #[get("/random")]
    async fn request_random(
        ctx: web::Data<AppState>,
        query: web::Query<Timestamp>,
    ) -> impl Responder {
        println!("Received random request from DApp timestamp={} version={}", query.timestamp, ctx.version);
        let randomness: Option<String> = ctx.get_randomness_for_timestamp(query.timestamp);
        if let Some(randomness) = randomness {
            // we already have the randomness to continue the process
            return HttpResponse::Ok().body(randomness);
        }
        // call finish to halt and wait the beacon
        let rollup_input = match send_finish_and_retrieve_input("accept").await {
            Some(input) => input,
            None => return HttpResponse::NotFound().finish(),
        };
        match rollup_input.request_type.as_str() {
            "advance_state" => {
                // Store the input in the buffer, so that it can be accessed from the /finish endpoint.
                ctx.store_input(&rollup_input).await;

                if let Some(beacon) = get_drand_beacon(&rollup_input.data.payload) {
                    println!("Is Drand!!! {:?}", beacon);
                    ctx.keep_newest_beacon(beacon);
                    let randomness = ctx.get_randomness_for_timestamp(query.timestamp);
                    if let Some(randomness) = randomness {
                        return HttpResponse::Ok().body(randomness);
                    }
                }
            }
            "inspect_state" => {
                if is_querying_pending_beacon(&rollup_input) {
                    send_pending_beacon_report(&ctx).await;

                    // This is a specific inspect, so we omit it from the DApp
                    return HttpResponse::NotFound().finish();
                } else {
                    // Store the input in the buffer, so that it can be accessed from the /finish endpoint.
                    ctx.store_input(&rollup_input).await;
                }
            }
            &_ => {
                eprintln!("Unknown request type");
            }
        };
        HttpResponse::NotFound().finish()
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
