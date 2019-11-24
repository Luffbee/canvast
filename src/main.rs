#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
use actix_web::{middleware, web, web::Data, App, HttpServer, Scope};

mod paint;
use paint::{now, PaintDB};
mod user;
use user::UserDB;

const APPNAME: &str = "CanVAST";

lazy_static! {
    static ref UDB: Data<user::SharedDB> = Data::new(user::SharedDB::new());
    static ref PDB: Data<paint::SharedDB> = Data::new(paint::SharedDB::new());
}

fn main() {
    // init the timer
    now();
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let addr = [([0, 0, 0, 0], 8088).into()];
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(api_v0(UDB.clone(), PDB.clone()))
    })
    .bind(&addr[..])
    .unwrap()
    .run()
    .unwrap();
}

fn api_v0<U, P>(udb: Data<U>, pdb: Data<P>) -> Scope
where
    U: UserDB + 'static,
    P: PaintDB + 'static,
{
    const VERSION: &str = "v0";
    web::scope(&format!("/{}", VERSION))
        .route("", web::get().to(|| format!("{} API {}", APPNAME, VERSION)))
        .route("/ping", web::get().to(|| "pong"))
        .service(
            web::scope("/user")
                .register_data(udb.clone())
                .configure(user::config::<U>),
        )
        .service(
            web::scope("/paint")
                .register_data(udb)
                .register_data(pdb)
                .configure(paint::config::<U, P>),
        )
}
