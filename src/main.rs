#![warn(unused_extern_crates)]
#![warn(clippy::dbg_macro)]
#![warn(missing_debug_implementations)]
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
use actix_web::{
    guard,
    middleware::Logger,
    put,
    web::{self, Json},
    App, HttpServer, Responder,
};
use log::{error, info};
use model::Report;
use serde::Deserialize;

use crate::{compare::Comparison, error::ApiResult};

const JSON_REPORTS_DIR: &str = "./output/json-reports/";
const HTML_REPORTS_DIR: &str = "./output/html-reports/";
const REPOSITORIES_DIR: &str = "./output/repositories/";
/// Name of the report used as comparison to calculate the difference in coverage
const DEFAULT_REPORT_NAME: &str = "main";

static DEFAULT_REPORT: LazyLock<RwLock<Option<Report>>> = LazyLock::new(|| RwLock::new(None));

#[derive(Debug, Deserialize)]
struct Request {
    name: String,
    git: String,
    branch: String,
    json_report: serde_json::Value,
}

#[put("")]
async fn new_report(request: Json<Request>) -> ApiResult<impl Responder> {
    info!(
        "Request {}, git: {}, branch: {}",
        request.name, request.git, request.branch
    );
    let report: Report = serde_json::from_value(request.json_report.clone())?;

    let repository_path = git::pull_or_clone(&request.git, &request.branch)?;

    // The working directory when the report was created, this need to be change with our path to the project.
    let raw_report = request.json_report.to_string();
    let outdated_working_directory = report
        .cargo_llvm_cov
        .manifest_path
        .trim_end_matches("/Cargo.toml");
    let raw_report_wd_replaced = raw_report.replace(
        outdated_working_directory,
        repository_path.to_str().unwrap(),
    );

    let json_path = PathBuf::from_str(JSON_REPORTS_DIR)
        .unwrap()
        .canonicalize()
        .unwrap()
        .join(&request.name);
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(json_path.clone())?;
    file.write_all(raw_report_wd_replaced.as_bytes())?;

    // Safety: the str is pre-defined
    let output_path = PathBuf::from_str(HTML_REPORTS_DIR)
        .unwrap()
        .canonicalize()
        .unwrap()
        .join(&request.name);

    let command = Command::new("llvm-cov-pretty")
        .current_dir(repository_path)
        .args([
            "--output-dir",
            output_path.to_str().unwrap(),
            "--manifest-path",
            "./Cargo.toml",
            json_path.to_str().unwrap(),
        ])
        .spawn()?
        .wait_with_output()?;
    if !command.status.success() {
        error!(
            "Error executing llvm-cov-pretty. code: {}\nstderr: {:#?}\nstdout: {:#?}",
            command.status, command.stderr, command.stdout
        );
        return Err(error::ApiError::LlvmCovPretty);
    }

    let comparison = {
        let default_report = DEFAULT_REPORT.read().expect("lock on default report");
        if let Some(default_report) = default_report.as_ref() {
            compare::function_coverage(&report, default_report)
        } else {
            Comparison::default()
        }
    };
    if request.name == DEFAULT_REPORT_NAME {
        let mut default_report = DEFAULT_REPORT.write().expect("lock on default report");
        *default_report = Some(report.clone());
    }

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
