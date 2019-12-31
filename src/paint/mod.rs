use actix_web::{
    error::Result,
    web,
    web::{Data, Json, Query},
    HttpRequest, HttpResponse,
};
use hex::FromHex;

use std::cmp::max;
use std::io::{Cursor, Seek, Write};

use crate::user::{authenticate, UserDB};

mod data;
pub use data::PixelPos;
use data::*;
mod db;
pub use db::{PaintDB, SharedDB};
mod error;
use error::InternalError;
pub use error::{PaintError, PaintResult};
mod line;
mod timestamp;
pub use timestamp::now;

pub fn config<P>(cfg: &mut web::ServiceConfig)
where
    P: PaintDB + 'static,
{
    cfg.route("/pixels", web::patch().to(draw_pixels::<P>))
        .route("/lines", web::patch().to(draw_lines::<P>))
        .service(
            web::resource("/blocks")
                .route(web::get().to(get_blocks::<P>))
                .route(web::patch().to(set_blocks::<P>)),
        )
        .service(
            web::resource("/locks")
                .route(web::get().to(get_locks))
                .route(web::post().to(set_locks))
                .route(web::delete().to(del_locks)),
        );
}

#[derive(Serialize)]
struct SuccessCount(#[serde(rename = "success_pixels")] usize);

#[derive(Deserialize)]
struct PixelsBody {
    color: String,
    base: PixelPos,
    offsets: Vec<Delta>,
}

impl PixelsBody {
    fn validate(&self) -> PaintResult<()> {
        const MAX_OFFSETS_NUM: usize = 1024;
        const MAX_OFFSET_ABS: i16 = 1024;

        if self.offsets.len() > MAX_OFFSETS_NUM {
            return Err(PaintError::InvalidData("too many offsets".to_owned()));
        }
        for offset in self.offsets.iter() {
            if !check_delta(*offset, -MAX_OFFSET_ABS, MAX_OFFSET_ABS) {
                return Err(PaintError::InvalidData("offset too large".to_owned()));
            }
        }
        Ok(())
    }
}

async fn draw_pixels<P: PaintDB>(
    udb: Data<UserDB>,
    pdb: Data<P>,
    req: HttpRequest,
    body: Json<PixelsBody>,
) -> Result<Json<SuccessCount>> {
    let color = RGBA::from_hex(&body.color)?;
    body.validate()?;
    let user = authenticate(&udb, &req).await?;
    let offsets = body.offsets.iter().map(|d| body.base + *d);
    Ok(Json(SuccessCount(pdb.draw_pixels(&user, color, offsets)?)))
}

#[derive(Deserialize)]
struct LinesBody {
    color: String,
    start: PixelPos,
    moves: Vec<Delta>,
}

impl LinesBody {
    fn validate(&self) -> PaintResult<()> {
        const MAX_MOVE_ABS: i16 = 2048;
        const MAX_MOVES_SUM: usize = 2048 * 4;

        let mut sum: usize = 0;
        for mv in self.moves.iter() {
            if !check_delta(*mv, -MAX_MOVE_ABS, MAX_MOVE_ABS) {
                return Err(PaintError::InvalidData("line segment too long".to_owned()));
            }
            sum += max(mv.x.abs(), mv.y.abs()) as usize;
            if sum > MAX_MOVES_SUM {
                return Err(PaintError::InvalidData("line too long".to_owned()));
            }
        }
        Ok(())
    }
}

async fn draw_lines<P: PaintDB>(
    udb: Data<UserDB>,
    pdb: Data<P>,
    req: HttpRequest,
    body: Json<LinesBody>,
) -> Result<Json<SuccessCount>> {
    let color = RGBA::from_hex(&body.color)?;
    body.validate()?;
    let user = authenticate(&udb, &req).await?;
    Ok(Json(SuccessCount(pdb.draw_lines(
        &user,
        color,
        body.start,
        body.moves.iter().copied(),
    )?)))
}

#[derive(Deserialize)]
struct RectTs {
    x: i64,
    y: i64,
    w: u8,
    h: u8,
    ts: u64,
}

async fn get_blocks<P: PaintDB>(pdb: Data<P>, Query(rect): Query<RectTs>) -> Result<HttpResponse> {
    let mut pngs = Vec::new();
    let base = BlockPos {
        x: rect.x,
        y: rect.y,
    };
    for i in 0..rect.w {
        for j in 0..rect.h {
            let mut data = Vec::<u8>::new();
            let ts = pdb.get_block(base + (i as u8, j as u8), Cursor::new(&mut data), rect.ts)?;
            if ts > rect.ts {
                let name = format!("{}_{}_{}.png", i, j, ts);
                pngs.push((name, data));
            }
        }
    }

    if pngs.is_empty() {
        return Ok(HttpResponse::NoContent().finish());
    }

    let mut payload = Vec::<u8>::new();
    zip_pngs(Cursor::new(&mut payload), pngs)?;
    Ok(HttpResponse::Ok()
        .content_type("application/zip")
        .body(payload))
}

async fn set_blocks<P: PaintDB>(
    _pdb: Data<P>,
    Query(_rect): Query<RectTs>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().finish())
}

fn zip_pngs<W: Write + Seek>(data: W, pngs: Vec<(String, Vec<u8>)>) -> Result<(), InternalError> {
    use zip::result::ZipError;
    let mut ziper = zip::ZipWriter::new(data);
    for (name, png) in pngs {
        let option = zip::write::FileOptions::default();
        ziper.start_file(name, option)?;
        ziper.write_all(&png).map_err(ZipError::from)?;
    }
    Ok(())
}

async fn get_locks(udb: Data<UserDB>, req: HttpRequest) -> Result<String> {
    Ok(authenticate(&udb, &req).await?)
}

async fn set_locks(_udb: Data<UserDB>, _req: HttpRequest) -> Result<&'static str> {
    Ok("set locks")
}

async fn del_locks(_udb: Data<UserDB>, _req: HttpRequest) -> Result<&'static str> {
    Ok("delete locks")
}

#[inline]
fn check_delta(d: Delta, min: i16, max: i16) -> bool {
    min <= d.x && d.x <= max && min <= d.y && d.y <= max
}
