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
mod storage;
#[cfg(test)]
mod tests;

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

use actix_files::Files;
use actix_web::{
    guard,
    middleware::Logger,
    put,
    web::{self, Data, Json},
    App, HttpServer, Responder,
};
use error::ApiError;
use log::{error, info};
use model::Report;
use serde::Deserialize;

use crate::{error::ApiResult, storage::TypedDb};

#[derive(Debug, Deserialize)]
struct Request {
    /// Url of the git repository associated to this report.
    ///
    /// We need to clone the repository so we can package the sources files into the export.
    git: String,
    /// Branch of git repository associated to this report.
    branch: String,
    /// The report generated when running `cargo llvm-cov --json`
    json_report: serde_json::Value,
}

impl Request {
    /// try to extract the project name from the git path, or return an url safe version of the git address
    fn repository_name(&self) -> String {
        let capture = config::REPOSITORY_REGEX.captures(&self.git);
        if let Some(name) = capture.map(|c| c.name("name").unwrap().as_str()) {
            name.replace('/', "-").to_lowercase()
        } else {
            utils::url_safe_string(&self.git)
        }
    }

    /// Based on the git url and the branch name
    fn unique_name(&self) -> String {
        let mut result = self.repository_name();
        result.push('-');
        result.push_str(&utils::url_safe_string(&self.branch));
        result
    }
}

fn find_matching_project_path<'a>(
    local_repository: &Path,
    remote_filepath: &'a str,
) -> ApiResult<&'a str> {
    let separators_positions: Vec<_> = remote_filepath
        .chars()
        .enumerate()
        .filter_map(|(idx, e)| (e == '/').then_some(idx))
        .collect();
    // Try to get an existing path from joining our local repository path with the one sent by the user
    // Start from the end of the filepath and for each subsequent tries take one more ancestors of the filepath
    let matching_project_path = separators_positions.iter().rev().find_map(|&sep| {
        let path = &remote_filepath[sep + 1..];
        local_repository
            .join(path)
            .exists()
            .then_some(&remote_filepath[..sep])
    });

    matching_project_path.ok_or(ApiError::FailedReportFilePathReplace)
}

/// Modify the report sources paths, with the path to the locally clone repository
///
/// In the case of a project that use workspaces, we need to find the root path first.
fn raw_report_with_local_repository(
    request: &Request,
    report: &Report,
    local_repository: &Path,
) -> ApiResult<String> {
    let report_data = report.data.first().ok_or(ApiError::NoReportData)?;
    // try to get a file cited in the report
    // we filter out any file containing the path "/.cargo/registry" to avoid dependency files
    let any_project_file_path = &report_data
        .files
        .iter()
        .find(|f| !f.filename.contains("/.cargo/registry"))
        .ok_or(ApiError::NoProjectFile)?
        .filename;
    let old_file_path = find_matching_project_path(local_repository, any_project_file_path)?;

    Ok(request
        .json_report
        .to_string()
        .replace(old_file_path, local_repository.to_str().unwrap()))
}

#[put("")]
async fn new_report(storage: Data<TypedDb>, request: Json<Request>) -> ApiResult<impl Responder> {
    info!("Request git: {}, branch: {}", request.git, request.branch);
    let report: Report = serde_json::from_value(request.json_report.clone())?;

    let repository_path = git::pull_or_clone(&request)?;

    let raw_report = raw_report_with_local_repository(&request, &report, &repository_path)?;

    // The working directory when the report was created, this need to be change with our path to the project.
    let json_path = PathBuf::from_str(config::JSON_REPORTS_DIR)
        .unwrap()
        .canonicalize()
        .unwrap()
        .join(request.unique_name());
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(json_path.clone())?;
    file.write_all(raw_report.as_bytes())?;

    // Safety: the str is pre-defined
    let output_path = PathBuf::from_str(config::HTML_REPORTS_DIR)
        .unwrap()
        .canonicalize()
        .unwrap()
        .join(request.unique_name());

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

    let comparison =
        compare::default_branch(&storage, &report, &request.branch).map_err(ApiError::from)?;
    info!(
        "Request git: {}, branch: {}: comparison: {:?}",
        request.git, request.branch, comparison
    );

    Ok(serde_json::to_string(&comparison))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    utils::init_environment();

    let api_key: &'static str =
        Box::leak(Box::new(std::env::var("API_KEY").expect("API_KEY in env")));
    let report_persistance = storage::TypedDb::new(sled::open(config::SLED_REPORTS_DIR)?);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new(
                r#"%{r}a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T"#,
            ))
            .app_data(web::JsonConfig::default().limit(1024 * 1024 * 100))
            .app_data(web::Data::new(report_persistance.clone()))
            .service(
                web::scope("/report")
                    .guard(guard::Header("x-api-key", api_key))
                    .service(new_report),
            )
            .service(Files::new("/view", config::HTML_REPORTS_DIR))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
