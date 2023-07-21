pub mod routes {
    use actix_web::{get, post, web, HttpResponse, Responder};
    use serde_json::json;
    use sha3::{Digest, Sha3_256};

    use crate::models::models::{AppState, RequestRollups, Timestamp};

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
        let manager = ctx.input_buffer_manager.try_lock();

        println!("Received finish request {:?}", body);

        // let mut counter = ctx.process_counter.lock().await;
        // *counter -= 1;

        let input = match manager {
            Ok(mut manager) => manager.consume_input(),
            Err(_) => return HttpResponse::NotFound().finish(),
        };

        match input {
            Some(item) => return HttpResponse::Ok().body(item.request),
            None => {}
        };
        let server_addr = std::env::var("ROLLUP_HTTP_SERVER_URL").unwrap();
        println!("Sending finish to {}", &server_addr);
        let client = hyper::Client::new();
        let status = "accept";
        let response = json!({"status" : status.clone()});
        let request = hyper::Request::builder()
            .method(hyper::Method::POST)
            .header(hyper::header::CONTENT_TYPE, "application/json")
            .uri(format!("{}/finish", &server_addr))
            .body(hyper::Body::from(response.to_string()))
            .unwrap();
        let response = client.request(request).await.unwrap();
        println!("Received finish status {}", response.status());
        if response.status() == hyper::StatusCode::ACCEPTED {
            return HttpResponse::Accepted().finish();
        }
        let body = hyper::body::to_bytes(response).await.unwrap();
        HttpResponse::Ok().body(body)
    }

    // GET /random?timestamp=123234
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
