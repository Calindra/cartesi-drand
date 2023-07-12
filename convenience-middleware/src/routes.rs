pub mod routes {
    use actix_web::{get, post, web, HttpResponse, Responder};
    use json::object;

    use crate::models::models::{AppState, RequestRollups};

    #[get("/")]
    async fn index() -> impl Responder {
        "Hello, World!"
    }

    #[post("/finish")]
    async fn consume_buffer(
        ctx: web::Data<AppState>,
        body: web::Json<RequestRollups>,
    ) -> impl Responder {
        let manager = ctx.input_buffer_manager.lock();

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

    #[get("/random")]
    async fn request_random(ctx: web::Data<AppState>) -> impl Responder {
        let mut manager = match ctx.input_buffer_manager.lock() {
            Ok(manager) => manager,
            Err(_) => return HttpResponse::BadRequest().finish(),
        };

        if manager.flag_to_hold.is_holding {
            return HttpResponse::Accepted().finish();
        } else {
            manager.flag_to_hold.hold_up();
        }

        // @todo Call service to request random

        manager.await_beacon();

        let data = "0x111111111111111111111";
        let json = object! { random: data };

        HttpResponse::Ok().json(json.to_string())
    }

    #[post("/hold")]
    async fn hold_buffer(ctx: web::Data<AppState>) -> impl Responder {
        let mut manager = match ctx.input_buffer_manager.lock() {
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
