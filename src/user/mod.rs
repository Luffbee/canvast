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
pub use db::UserDB;

mod error;
pub use error::{UserError, UserResult};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::post().to(register))
        .service(
            web::resource("/auth")
                .route(web::post().to(login))
                .route(web::delete().to(logout)),
        )
        .service(
            web::resource("/location")
                .route(web::get().to(get_location))
                .route(web::put().to(set_location)),
        );
}

async fn register(db: Data<UserDB>, user: Json<WithPassword>) -> Result<impl Responder> {
    user.validate()?;
    db.new_user(user.into_inner()).await?;
    Ok(HttpResponse::Ok())
}

const TOKEN_NAME: &str = "CanVastAuthToken";

async fn login(db: Data<UserDB>, user: Json<WithPassword>) -> Result<impl Responder> {
    let (token, exp) = db.login(&user).await?;
    let cookie = Cookie::build(TOKEN_NAME, token)
        .path("/")
        .secure(true)
        .http_only(true)
        .expires(exp)
        .finish();
    Ok(web::HttpResponse::Ok().cookie(cookie).finish())
}

async fn logout(db: Data<UserDB>, req: HttpRequest) -> Result<impl Responder> {
    let cookie = get_cookie(&req)?;
    db.logout(cookie.value()).await?;
    Ok(HttpResponse::Ok())
}

async fn set_location(
    db: Data<UserDB>,
    req: HttpRequest,
    loc: Json<PixelPos>,
) -> Result<impl Responder> {
    let name = authenticate(&db, &req).await?;
    db.set_location(name, loc.into_inner()).await?;
    Ok(HttpResponse::Ok())
}

async fn get_location(db: Data<UserDB>, req: HttpRequest) -> Result<Json<PixelPos>> {
    let name = authenticate(&db, &req).await?;
    let loc = db.get_location(&name).await?;
    Ok(Json(loc))
}

fn get_cookie(req: &HttpRequest) -> Result<Cookie<'static>> {
    Ok(req.cookie(TOKEN_NAME).ok_or(UserError::NoToken)?)
}

pub async fn authenticate(db: &Data<UserDB>, req: &HttpRequest) -> Result<Username> {
    let cookie = get_cookie(req)?;
    let name = db.check_token(cookie.value()).await?;
    Ok(name)
}
