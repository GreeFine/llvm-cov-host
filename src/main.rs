#![warn(unused_crate_dependencies)]
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
    path::{Path, PathBuf},
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
use error::ApiError;
use log::{error, info};
use model::Report;
use regex::{self, Regex};
use serde::Deserialize;

use crate::{compare::Comparison, error::ApiResult};

const JSON_REPORTS_DIR: &str = "./output/json-reports/";
const HTML_REPORTS_DIR: &str = "./output/html-reports/";
const REPOSITORIES_DIR: &str = "./output/repositories/";
/// Name of the report used as comparison to calculate the difference in coverage
const DEFAULT_REPORT_NAME: &str = "main";

static DEFAULT_REPORT: LazyLock<RwLock<Option<Report>>> = LazyLock::new(|| RwLock::new(None));
static REPOSITORY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(.*@.*:|https:\/\/.*\.*.\/)(?<name>[A-z-]{0,100}\/[A-z-]{0,100})\.git"#)
        .expect("valid regex for repository")
});

#[derive(Debug, Deserialize)]
struct Request {
    /// Url of the git repository associated to this report.
    ///
    /// We need to clone the repository so we can package the sources into the export.
    git: String,
    /// Branch of git repository associated to this report.
    branch: String,
    /// The report generated when running `cargo llvm-cov --json`
    json_report: serde_json::Value,
}

impl Request {
    /// try to extract the project name from the git path, or return an url safe version of the git address
    fn name(&self) -> String {
        let capture = REPOSITORY_REGEX.captures(&self.git);
        if let Some(name) = capture.map(|c| c.name("name").unwrap().as_str()) {
            name.to_string()
        } else {
            let mut prev_is_dash = false;
            let name_safe: String = self
                .git
                .chars()
                .filter_map(|c| {
                    if c.is_ascii_alphanumeric() {
                        prev_is_dash = false;
                        Some(c.to_ascii_lowercase())
                    } else {
                        (!prev_is_dash).then(|| {
                            prev_is_dash = true;
                            '-'
                        })
                    }
                })
                .collect();
            name_safe
        }
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
async fn new_report(request: Json<Request>) -> ApiResult<impl Responder> {
    info!("Request git: {}, branch: {}", request.git, request.branch);
    let report: Report = serde_json::from_value(request.json_report.clone())?;

    let repository_path = git::pull_or_clone(&request)?;

    let raw_report = raw_report_with_local_repository(&request, &report, &repository_path)?;

    // The working directory when the report was created, this need to be change with our path to the project.
    let json_path = PathBuf::from_str(JSON_REPORTS_DIR)
        .unwrap()
        .canonicalize()
        .unwrap()
        .join(request.name());
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(json_path.clone())?;
    file.write_all(raw_report.as_bytes())?;

    // Safety: the str is pre-defined
    let output_path = PathBuf::from_str(HTML_REPORTS_DIR)
        .unwrap()
        .canonicalize()
        .unwrap()
        .join(request.name());

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
    if request.name() == DEFAULT_REPORT_NAME {
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

#[test]
fn test_get_name() {
    let request = Request {
        branch: "main/aqwqe/2".to_string(),
        git: "https://github.com/GreeFine/llvm-cov-host".to_string(),
        json_report: serde_json::Value::Null,
    };
    assert_eq!(
        request.name(),
        "https-github-com-GreeFine-llvm-cov-host-main-aqwqe-2"
    );
}

#[test]
fn test_raw_report_with_local_repository() {
    use std::fs::{self, File};

    let dir = Path::new("/tmp/test-llvm-cov-host/api/src");
    fs::create_dir_all(dir).unwrap();
    File::create(dir.join("compare.rs")).unwrap();

    let local_repository = Path::new("/tmp/test-llvm-cov-host/");
    let remote_filepath = "/home/greefine/Projects/llvm-cov-host/api/src/compare.rs";
    let old_path = find_matching_project_path(local_repository, remote_filepath).unwrap();

    fs::remove_dir_all("/tmp/test-llvm-cov-host/").unwrap();

    assert_eq!(old_path, "/home/greefine/Projects/llvm-cov-host");
}
