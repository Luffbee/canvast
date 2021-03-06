use actix_web::{middleware, web, web::Data, App, HttpServer, Scope};
use futures::future::ready;
use lazy_static::lazy_static;

mod paint;
use paint::{now, PaintDB};
mod user;
use user::UserDB;

const APPNAME: &str = "CanVAST";

lazy_static! {
    static ref UDB: Data<UserDB> = Data::new(UserDB::new());
    static ref PDB: Data<PaintDB> = Data::new(PaintDB::new());
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    // init the timer
    now();
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "canvast=warn,actix_web=info");
    }
    env_logger::init();

    let addr = [([0, 0, 0, 0], 8088).into()];
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(api_v0(UDB.clone(), PDB.clone()))
    })
    .bind(&addr[..])?
    .run()
    .await
}

fn api_v0(udb: Data<UserDB>, pdb: Data<PaintDB>) -> Scope
where
{
    const VERSION: &str = "v0";
    web::scope(&format!("/{}", VERSION))
        .route(
            "",
            web::get().to(|| ready(format!("{} API {}", APPNAME, VERSION))),
        )
        .route("/ping", web::get().to(|| ready("pong")))
        .service(
            web::scope("/user")
                .app_data(udb.clone())
                .configure(user::config),
        )
        .service(
            web::scope("/paint")
                .app_data(udb)
                .app_data(pdb)
                .configure(paint::config),
        )
}
