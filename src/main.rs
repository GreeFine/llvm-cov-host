#![warn(unused_crate_dependencies)]
#![warn(clippy::dbg_macro)]
#![warn(missing_debug_implementations)]
#![feature(lazy_cell)]

mod compare;
mod error;
mod git;
mod model;
mod utils;

mod config;
mod dashboard;
mod report;
mod routes;
mod storage;
#[cfg(test)]
mod tests;

use actix_files::Files;
use actix_web::{
    guard,
    middleware::Logger,
    web::{self},
    App, HttpServer,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    utils::init_environment();

    let api_key: &'static str =
        Box::leak(Box::new(std::env::var("API_KEY").expect("API_KEY in env")));
    let report_persistance = storage::TypedDb::new(sled::open(config::SLED_DIR)?);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new(
                r#"%{r}a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T"#,
            ))
            .app_data(web::JsonConfig::default().limit(1024 * 1024 * 1000))
            .app_data(web::Data::new(report_persistance.clone()))
            .service(web::scope("/").service(routes::dashboard))
            .service(
                web::scope("/report")
                    .guard(guard::Header("x-api-key", api_key))
                    .service(routes::new_report),
            )
            .service(Files::new("/view", config::HTML_REPORTS_DIR))
            .service(
                Files::new("/css", "templates/")
                    .path_filter(|p, _| p.extension().map(|e| e == "css") == Some(true)),
            )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
