#![feature(lazy_cell)]
mod compare;
mod error;
mod git;
mod model;
mod utils;

#[cfg(test)]
mod tests;

use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::Command,
    str::FromStr,
    sync::{LazyLock, RwLock},
};

use actix_files::Files;
use actix_web::{guard, middleware::Logger, put, web, App, HttpServer, Responder};
use model::Report;
use serde::Deserialize;

use crate::{compare::Comparison, error::ApiResult};

const JSON_REPORTS_DIR: &str = "./json-reports/";
const HTML_REPORTS_DIR: &str = "./html-reports/";
/// Name of the report used as comparison to calculate the difference in coverage
const DEFAULT_REPORT_NAME: &str = "main";

static DEFAULT_REPORT: LazyLock<RwLock<Option<Report>>> = LazyLock::new(|| RwLock::new(None));

#[derive(Debug, Deserialize)]
struct NewReport {
    name: String,
    git: String,
    report_json: Report,
    branch: String,
}

#[put("/")]
async fn new_report(request: web::Json<NewReport>) -> ApiResult<impl Responder> {
    let mut path = PathBuf::new();
    path.push(JSON_REPORTS_DIR);
    path.push(&request.name);
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path.clone())?;

    let comparison = {
        let default_report = DEFAULT_REPORT.read().expect("lock on default report");
        if let Some(default_report) = default_report.as_ref() {
            compare::function_coverage(&request.report_json, default_report)
        } else {
            Comparison::default()
        }
    };
    if request.name == DEFAULT_REPORT_NAME {
        let mut default_report = DEFAULT_REPORT.write().expect("lock on default report");
        *default_report = Some(request.report_json.clone());
    }

    let repository_path = git::pull_or_clone(&request.git, &request.branch)?;

    file.write_all(&serde_json::to_vec(&request.report_json).unwrap())?;
    // Safety: the str is pre-defined
    let mut output_path = PathBuf::from_str(HTML_REPORTS_DIR).unwrap();
    output_path.push(&request.name);
    Command::new("llvm-cov-pretty")
        .args([
            "--output-dir",
            output_path.to_str().unwrap(),
            "--manifest-path",
            repository_path.to_str().unwrap(),
            path.to_str().unwrap(),
        ])
        .spawn()?;

    Ok(serde_json::to_string(&comparison))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    utils::init_environment();

    let api_key: &'static str =
        Box::leak(Box::new(std::env::var("API_KEY").expect("API_KEY in env")));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new(
                r#"%{r}a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T"#,
            ))
            .app_data(web::JsonConfig::default().limit(1024 * 1024 * 100))
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
