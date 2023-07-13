pub mod routes {
    use actix_web::{get, post, web, HttpResponse, Responder};

    use crate::models::models::{AppState, RequestRollups, Beacon, Timestamp};

    #[get("/")]
    async fn index() -> impl Responder {
        "Hello, World!"
    }

    #[post("/finish")]
    async fn consume_buffer(
        ctx: web::Data<AppState>,
        body: web::Json<RequestRollups>,
    ) -> impl Responder {
        let manager = ctx.input_buffer_manager.try_lock();

        println!("Received finish request {:?}", body);

        let input = match manager {
            Ok(mut manager) => manager.consume_input(),
            Err(_) => return HttpResponse::NotFound().finish(),
        };

        match input {
            Some(item) => {
                let parse = json::parse(&item.request).unwrap();
                HttpResponse::Ok().body(parse.to_string())
            }
            None => HttpResponse::Accepted().finish(),
        }
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

        // temos que pensar melhor o hold para identificar o request que inicial e deixar ele passar
        // if manager.flag_to_hold.is_holding {
        //     return HttpResponse::NotFound().into();
        // } else {
        //     manager.flag_to_hold.hold_up();
        // }
        let res = match manager.last_beacon.take() {
            Some(beacon) => {
                println!("beacon time {}", beacon.timestamp);
                // comparamos se o beacon é suficientemente velho pra devolver como resposta
                if query.timestamp < beacon.timestamp - 3 {
                    // @todo retornar apenas o randomness
                    let resp = HttpResponse::Ok().json(beacon.metadata.to_owned());
                    manager.flag_to_hold.release();
                    manager.last_beacon.set(Some(beacon));
                    resp
                } else {
                    manager.set_pending_beacon_timestamp(query.timestamp);
                    manager.last_beacon.set(Some(beacon));
                    HttpResponse::NotFound().into()
                }
            }
            None => {
                manager.set_pending_beacon_timestamp(query.timestamp);

                // @todo somente para validar, remover depois
                manager.last_beacon.set(Some(Beacon {
                    timestamp: 123,
                    metadata: "batata".to_string()
                }));
                HttpResponse::NotFound().into()
            }
        };
        res
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
