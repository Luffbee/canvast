use actix_web::{
    error::Result,
    http::Cookie,
    web,
    web::{Data, Json},
    HttpMessage, HttpRequest, Responder,
};

mod data;
pub use data::User;
use data::*;

mod db;
pub use db::{SharedDB, DB};

mod error;
pub use error::{UserError, UserResult};

pub fn config<T: DB + 'static>(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user")
            .route("", web::post().to(register::<T>))
            .service(
                web::resource("/auth")
                    .route(web::post().to(login::<T>))
                    .route(web::delete().to(logout::<T>)),
            )
            .service(
                web::resource("/location")
                    .route(web::get().to(get_location::<T>))
                    .route(web::put().to(set_location::<T>)),
            ),
    );
}

fn register<T: DB>(db: Data<T>, user: Json<WithPassword>) -> Result<()> {
    user.validate()?;
    db.new_user(user.into_inner())?;
    Ok(())
}

const TOKEN_NAME: &str = "CanVastAuthToken";

fn login<T: DB>(db: Data<T>, user: Json<WithPassword>) -> Result<impl Responder> {
    let (token, exp) = db.login(&user)?;
    let cookie = Cookie::build(TOKEN_NAME, token)
        .path("/")
        .secure(true)
        .http_only(true)
        .expires(exp)
        .finish();
    Ok(web::HttpResponse::Ok().cookie(cookie).finish())
}

fn logout<T: DB>(db: Data<T>, req: HttpRequest) -> Result<()> {
    let cookie = req.cookie(TOKEN_NAME).ok_or(UserError::NoToken)?;
    db.logout(cookie.value())?;
    Ok(())
}

fn set_location<T: DB>(db: Data<T>, req: HttpRequest, loc: Json<Location>) -> Result<()> {
    loc.validate()?;
    let name = get_user(&db, &req)?;
    db.set_location(name, loc.into_inner())?;
    Ok(())
}

fn get_location<T: DB>(db: Data<T>, req: HttpRequest) -> Result<Json<Location>> {
    let name = get_user(&db, &req)?;
    let loc = db.get_location(&name)?;
    Ok(Json(loc))
}

fn get_user<T: DB>(db: &Data<T>, req: &HttpRequest) -> Result<Username> {
    let cookie = req.cookie(TOKEN_NAME).ok_or(UserError::NoToken)?;
    let name = db.check_token(cookie.value())?;
    Ok(name)
}
