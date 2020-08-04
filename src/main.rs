#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate redis_async;

use crate::routes::auth;
use crate::util::{auth::get_identity_service, cache::add_cache, config::CONFIG, state::new_state};
use actix_web::{middleware::Logger, web, App, HttpServer, Result};
use actix_files::NamedFile;
use env_logger::Env;

mod middleware;
mod routes;
mod util;

async fn file(file: &str) -> Result<NamedFile> {
    Ok(NamedFile::open(format!("./{}.html", file))?)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::from_env(Env::default().default_filter_or("debug")).init();
    info!("Starting...");

    let data = new_state::<String>();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(get_identity_service())
            .configure(add_cache)
            .app_data(data.clone())
            .route("/", web::to(|| file("index")))
            .service(web::scope("/").configure(auth::init))
            .default_service(web::to(|| file("notfound")))
    })
    .bind(&CONFIG.server_address)?
    .run()
    .await
}
