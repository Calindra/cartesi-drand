pub mod routes {
    use actix_web::{get, post, web, HttpResponse, Responder};

    use crate::models::models::AppState;

    #[get("/")]
    async fn index() -> impl Responder {
        "Hello, World!"
    }

    #[get("/consume")]
    async fn consume_buffer(ctx: web::Data<AppState>) -> HttpResponse {
        let manager = ctx.input_buffer_manager.lock();

        let input = match manager {
            Ok(mut manager) => manager.consume_input(),
            Err(_) => return HttpResponse::BadRequest().finish(),
        };

        match input {
            Some(item) => HttpResponse::Ok().body(item.request),
            None => HttpResponse::NoContent().finish(),
        }
    }

    #[post("/hold")]
    async fn hold_buffer(ctx: web::Data<AppState>) -> HttpResponse {
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
