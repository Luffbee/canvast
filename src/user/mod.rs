use actix_web::{
    http::{Cookie, StatusCode},
    web,
    web::{Data, Json},
    Either, HttpMessage, HttpRequest, Responder,
};

mod data;
pub use data::User;
use data::*;

mod db;
pub use db::{SharedDB, DB};

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

#[derive(Serialize)]
struct FailedReason {
    reason: String,
}

type ReasonStatus = (Json<FailedReason>, StatusCode);

fn reason_status(reason: String, st: StatusCode) -> ReasonStatus {
    (Json(FailedReason { reason }), st)
}

fn register<T: DB>(db: Data<T>, user: Json<WithPassword>) -> impl Responder {
    match user.validate() {
        Err(reason) => Either::B((
            Json(FailedReason { reason }),
            StatusCode::UNPROCESSABLE_ENTITY,
        )),
        Ok(()) => match db.new_user(user.into_inner()) {
            Err(_) => Either::A(((), StatusCode::CONFLICT)),
            Ok(()) => Either::A(((), StatusCode::OK)),
        },
    }
}

const TOKEN_NAME: &str = "CanVastAuthToken";

fn login<T: DB>(db: Data<T>, user: Json<WithPassword>) -> impl Responder {
    match db.login(&user) {
        Err(reason) => Either::B((Json(FailedReason { reason }), StatusCode::UNAUTHORIZED)),
        Ok((token, exp)) => {
            let cookie = Cookie::build(TOKEN_NAME, token)
                .path("/")
                .secure(true)
                .http_only(true)
                .expires(exp)
                .finish();
            Either::A(web::HttpResponse::Ok().cookie(cookie).finish())
        }
    }
}

fn logout<T: DB>(db: Data<T>, req: HttpRequest) -> impl Responder {
    match req.cookie(TOKEN_NAME) {
        None => Either::B(reason_status(
            "no token".to_owned(),
            StatusCode::UNAUTHORIZED,
        )),
        Some(cookie) => match db.logout(cookie.value()) {
            Err(e) => Either::B(reason_status(e, StatusCode::UNAUTHORIZED)),
            Ok(()) => Either::A(()),
        },
    }
}

fn set_location<T: DB>(db: Data<T>, req: HttpRequest, loc: Json<Location>) -> impl Responder {
    match get_user(&db, &req) {
        Err(e) => Either::B(e),
        Ok(name) => match loc.validate() {
            Err(e) => Either::B(reason_status(e, StatusCode::UNPROCESSABLE_ENTITY)),
            Ok(()) => match db.set_location(name, loc.into_inner()) {
                // FIXME: handle 500
                Err(e) => Either::B(reason_status(e, StatusCode::INTERNAL_SERVER_ERROR)),
                Ok(()) => Either::A(()),
            },
        },
    }
}

fn get_location<T: DB>(db: Data<T>, req: HttpRequest) -> impl Responder {
    match get_user(&db, &req) {
        Err(e) => Either::B(e),
        Ok(name) => match db.get_location(&name) {
            // FIXME: handle 500
            Err(e) => Either::B(reason_status(e, StatusCode::INTERNAL_SERVER_ERROR)),
            Ok(loc) => Either::A(Json(loc)),
        },
    }
}

fn get_user<T: DB>(db: &Data<T>, req: &HttpRequest) -> Result<Username, ReasonStatus> {
    match req.cookie(TOKEN_NAME) {
        None => Err(reason_status(
            "no token".to_owned(),
            StatusCode::UNAUTHORIZED,
        )),
        Some(cookie) => match db.check_token(cookie.value()) {
            Err(e) => Err(reason_status(e, StatusCode::UNAUTHORIZED)),
            Ok(name) => Ok(name),
        },
    }
}
