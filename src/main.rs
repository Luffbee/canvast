#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
use actix_web::{middleware, web, web::Data, App, HttpServer, Scope};

mod paint;
mod user;
use user::UserDB;

const APPNAME: &str = "CanVAST";

fn main() {
    let addr = [([127, 0, 0, 1], 8088).into()];
    lazy_static! {
        static ref USER_DB: Data<user::SharedDB> = Data::new(user::SharedDB::new());
    }
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(api_v1(USER_DB.clone()))
    })
    .bind(&addr[..])
    .unwrap()
    .run()
    .unwrap();
}

fn api_v1<U: user::UserDB + 'static>(user_db: Data<U>) -> Scope {
    const VERSION: &str = "v1";
    web::scope(&format!("/{}", VERSION))
        .register_data(user_db)
        .route("", web::get().to(|| format!("{} API {}", APPNAME, VERSION)))
        .route("/ping", web::get().to(|| "pong"))
        .configure(user::config::<U>)
        .configure(paint::config)
}

/*fn ziptest() {
    use zip;
    use rand::random;
    use std::path::Path;
    use std::fs::File;
    use std::io::{Write, BufWriter};

    let zipfile = File::create(Path::new("blocks_13A_-9F_5_5.zip")).unwrap();
    let ref mut w = BufWriter::new(zipfile);
    let mut ziper = zip::ZipWriter::new(w);

    for i in 0..5 {
        for j in 0..5 {
            let name = format!("block_{}_{}.png", i, j);

            let options = zip::write::FileOptions::default();
            ziper.start_file(&name, options).unwrap();

            const BLOCKSIZE: u32 = 64;
            let mut encoder = png::Encoder::new(ziper.by_ref(), BLOCKSIZE, BLOCKSIZE);
            encoder.set_color(png::ColorType::RGB);
            encoder.set_depth(png::BitDepth::Eight);
            let mut write = encoder.write_header().unwrap();

            let mut data: Vec<u8> = Vec::new();
            for _ in 0..BLOCKSIZE {
                for _ in 0..BLOCKSIZE {
                    for _ in 0..3 {
                        data.push(random::<u8>());
                    }
                }
            }
            write.write_image_data(&data).unwrap();
        }
    }
}*/
