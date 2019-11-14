use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/paint")
            .route("/pixels", web::put().to(|| "set pixels"))
            .service(
                web::resource("/blocks")
                    .route(web::get().to(|| "get blocks"))
                    .route(web::put().to(|| "set blocks")),
            )
            .service(
                web::resource("/locks")
                    .route(web::get().to(|| "get locks"))
                    .route(web::post().to(|| "create lock"))
                    .route(web::delete().to(|| "delete lock")),
            ),
    );
}
