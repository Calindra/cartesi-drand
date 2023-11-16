pub mod routes {
    use actix_web::{get, post, put, web, HttpResponse, Responder};
    use log::{error, info};

    use crate::{
        drand::{get_drand_beacon, is_querying_pending_beacon, send_pending_beacon_report},
        models::models::{AppState, DrandEnv, RequestRollups, Timestamp},
        rollup::{has_input_inside_input, server::send_finish_and_retrieve_input},
        utils::util::{load_env_from_memory, write_env_to_json},
    };

    #[put("/update_drand_config")]
    async fn update_drand_config(
        ctx: web::Data<AppState>,
        body: web::Json<DrandEnv>,
    ) -> impl Responder {
        info!(
            "Received update_drand_config request from DApp version={}",
            ctx.version
        );

        let _ = ctx.input_buffer_manager.lock().await;

        let drand = body.into_inner();

        load_env_from_memory(drand).await;

        let result = write_env_to_json().await;

        // maybe can generate a error on write json but
        // already change the env in memory
        if let Err(e) = result {
            error!("Error updating drand config: {}", e);
            return HttpResponse::BadRequest().finish();
        }

        HttpResponse::NoContent().finish()
    }

    #[post("/finish")]
    async fn consume_buffer(
        ctx: web::Data<AppState>,
        body: web::Json<RequestRollups>,
    ) -> impl Responder {
        info!(
            "Received finish request from DApp {:?} version={}",
            body, ctx.version
        );

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
                ctx.set_inspecting(false).await;
                if let Some(beacon) = get_drand_beacon(&rollup_input.data.payload) {
                    info!("Is Drand!!! {:?}", beacon);
                    ctx.keep_newest_beacon(beacon);
                }
            }
            "inspect_state" => {
                ctx.set_inspecting(true).await;
                if is_querying_pending_beacon(&rollup_input) {
                    send_pending_beacon_report(&ctx).await;

                    // This is a specific inspect, so we omit it from the DApp
                    return HttpResponse::Accepted().finish();
                }
            }
            &_ => {
                error!("Unknown request type");
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
        info!(
            "Received random request from DApp timestamp={} version={}",
            query.timestamp, ctx.version
        );
        let randomness: Option<String> = ctx.get_randomness_for_timestamp(query.timestamp);
        if let Some(randomness) = randomness {
            // we already have the randomness to continue the process
            return HttpResponse::Ok().body(randomness);
        }
        if ctx.is_inspecting() {
            info!("When inspecting we does not call finish from /random endpoint.");
            return HttpResponse::BadRequest().finish();
        }
        // call finish to halt and wait the beacon
        let rollup_input = match send_finish_and_retrieve_input("accept").await {
            Some(input) => input,
            None => return HttpResponse::NotFound().finish(),
        };
        match rollup_input.request_type.as_str() {
            "advance_state" => {
                ctx.set_inspecting(false).await;
                // Store the input in the buffer, so that it can be accessed from the /finish endpoint.
                ctx.store_input(&rollup_input).await;

                if let Some(beacon) = get_drand_beacon(&rollup_input.data.payload) {
                    info!("Is Drand!!! {:?}", beacon);
                    ctx.keep_newest_beacon(beacon);
                    let randomness = ctx.get_randomness_for_timestamp(query.timestamp);
                    if let Some(randomness) = randomness {
                        return HttpResponse::Ok().body(randomness);
                    }
                }
            }
            "inspect_state" => {
                ctx.set_inspecting(true).await;
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
                error!("Unknown request type");
            }
        };
        HttpResponse::NotFound().finish()
    }
}
