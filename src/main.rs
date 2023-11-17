#![feature(lazy_cell)]
mod compare;
mod error;
mod model;

use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::Command,
    str::FromStr,
    sync::{LazyLock, RwLock},
};

use actix_files::Files;
use actix_web::{guard, put, web, App, HttpServer, Responder};
use model::Report;

use crate::{compare::Comparison, error::ApiResult};

const JSON_REPORTS_DIR: &str = "./json-reports/";
const HTML_REPORTS_DIR: &str = "./html-reports/";
/// Name of the report used as comparison to calculate the difference in coverage
const DEFAULT_REPORT_NAME: &str = "main";

static DEFAULT_REPORT: LazyLock<RwLock<Option<Report>>> = LazyLock::new(|| RwLock::new(None));

#[put("/{name}")]
async fn new_report(
    name: web::Path<String>,
    json_content: web::Bytes,
) -> ApiResult<impl Responder> {
    let name = &name.into_inner();
    let mut path = PathBuf::new();
    path.push(JSON_REPORTS_DIR);
    path.push(name);
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path.clone())?;

    // Check that is look like a valid report
    let report: Report = serde_json::from_slice(&json_content)?;

    let comparison = {
        let default_report = DEFAULT_REPORT.read().expect("lock on default report");
        if let Some(default_report) = default_report.as_ref() {
            compare::function_coverage(&report, default_report)
        } else {
            Comparison::default()
        }
    };
    if name == DEFAULT_REPORT_NAME {
        let mut default_report = DEFAULT_REPORT.write().expect("lock on default report");
        *default_report = Some(report);
    }

    file.write_all(&json_content)?;

    // Safety: the str is pre-defined
    let mut output_path = PathBuf::from_str(HTML_REPORTS_DIR).unwrap();
    output_path.push(name);
    Command::new("llvm-cov-pretty")
        .args([
            "--output-dir",
            output_path.to_str().unwrap(),
            path.to_str().unwrap(),
        ])
        .spawn()?;

    Ok(serde_json::to_string(&comparison))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let api_key: &'static str =
        Box::leak(Box::new(std::env::var("API_KEY").expect("API_KEY in env")));

    HttpServer::new(move || {
        App::new()
            .app_data(web::PayloadConfig::new(1024 * 1024 * 100))
            .service(
                web::scope("/report")
                    .guard(guard::Header("x-api-key", api_key))
                    .service(new_report),
            )
            .service(Files::new("/view", HTML_REPORTS_DIR))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
