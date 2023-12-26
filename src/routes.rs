use std::{path::PathBuf, process::Command, str::FromStr};

use actix_web::{get, http::StatusCode, put, web, HttpResponse, Responder};
use askama::Template;
use chrono::Utc;
use log::{error, info};

use crate::{
    compare, config,
    dashboard::DashBoardTemplate,
    error::ApiError,
    git,
    model::Report,
    report::{self, ReportHistory, Request},
    storage::TypedDb,
};

#[put("")]
pub async fn new_report(
    storage: web::Data<TypedDb>,
    request: web::Json<Request>,
) -> impl Responder {
    info!("Request git: {}, branch: {}", request.git, request.branch);
    let report: Report = serde_json::from_value(request.json_report.clone())?;

    let repository_path = git::pull_or_clone(&request)?;
    let json_path = report::save_json(&request, &report, &repository_path)?;

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
            &json_path,
        ])
        .spawn()?
        .wait_with_output()?;
    if !command.status.success() {
        error!(
            "Error executing llvm-cov-pretty. code: {}\nstderr: {:#?}\nstdout: {:#?}",
            command.status, command.stderr, command.stdout
        );
        return Err(ApiError::LlvmCovPretty);
    }

    let comparison =
        compare::default_branch(&storage, &report, &request.branch).map_err(ApiError::from)?;
    info!(
        "Request git: {}, branch: {}: comparison: {:?}",
        request.git, request.branch, comparison
    );
    let now = Utc::now().naive_utc();
    storage.insert(
        &now.to_string(),
        &ReportHistory {
            branch: request.branch.clone(),
            repository_name: request.raw_repository_name(),
            name: request.unique_name(),
            comparison: comparison.clone(),
            date: now,
        },
    )?;

    Ok(serde_json::to_string(&comparison))
}

#[get("")]
pub async fn dashboard(storage: web::Data<TypedDb>) -> impl Responder {
    let reports: Vec<ReportHistory> = storage.get_all().map_err(ApiError::from)?;
    let page = DashBoardTemplate { reports };

    Ok::<HttpResponse, ApiError>(
        HttpResponse::build(StatusCode::OK)
            .content_type("text/html; charset=utf-8")
            .body(page.render().unwrap()),
    )
}
