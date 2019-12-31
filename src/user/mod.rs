use actix_web::{
    error::Result,
    http::Cookie,
    web,
    web::{Data, Json},
    HttpMessage, HttpRequest, HttpResponse, Responder,
};

use crate::paint::PixelPos;

mod data;
use data::*;
pub use data::{User, Username};

mod db;
pub use db::{SharedDB, UserDB};

mod error;
pub use error::{UserError, UserResult};

pub fn config<T: UserDB + 'static>(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::post().to(register::<T>))
        .service(
            web::resource("/auth")
                .route(web::post().to(login::<T>))
                .route(web::delete().to(logout::<T>)),
        )
        .service(
            web::resource("/location")
                .route(web::get().to(get_location::<T>))
                .route(web::put().to(set_location::<T>)),
        );
}

async fn register<T: UserDB>(db: Data<T>, user: Json<WithPassword>) -> Result<impl Responder> {
    user.validate()?;
    db.new_user(user.into_inner())?;
    Ok(HttpResponse::Ok())
}

const TOKEN_NAME: &str = "CanVastAuthToken";

async fn login<T: UserDB>(db: Data<T>, user: Json<WithPassword>) -> Result<impl Responder> {
    let (token, exp) = db.login(&user)?;
    let cookie = Cookie::build(TOKEN_NAME, token)
        .path("/")
        .secure(true)
        .http_only(true)
        .expires(exp)
        .finish();
    Ok(web::HttpResponse::Ok().cookie(cookie).finish())
}

async fn logout<T: UserDB>(db: Data<T>, req: HttpRequest) -> Result<impl Responder> {
    let cookie = get_cookie(&req)?;
    db.logout(cookie.value())?;
    Ok(HttpResponse::Ok())
}

async fn set_location<T: UserDB>(
    db: Data<T>,
    req: HttpRequest,
    loc: Json<PixelPos>,
) -> Result<impl Responder> {
    let name = authenticate(&db, &req).await?;
    db.set_location(name, loc.into_inner())?;
    Ok(HttpResponse::Ok())
}

async fn get_location<T: UserDB>(db: Data<T>, req: HttpRequest) -> Result<Json<PixelPos>> {
    let name = authenticate(&db, &req).await?;
    let loc = db.get_location(&name)?;
    Ok(Json(loc))
}

fn get_cookie(req: &HttpRequest) -> Result<Cookie<'static>> {
    Ok(req.cookie(TOKEN_NAME).ok_or(UserError::NoToken)?)
}

pub async fn authenticate<T: UserDB>(db: &Data<T>, req: &HttpRequest) -> Result<Username> {
    let cookie = get_cookie(req)?;
    let name = db.check_token(cookie.value())?;
    Ok(name)
}
