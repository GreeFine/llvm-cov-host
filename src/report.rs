use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    str::FromStr,
};

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::{
    compare::Comparison,
    config,
    error::{ApiError, ApiResult},
    model::Report,
    utils,
};

#[derive(Debug, Deserialize)]
pub struct Request {
    /// Url of the git repository associated to this report.
    ///
    /// We need to clone the repository so we can package the sources files into the export.
    pub git: String,
    /// Branch of git repository associated to this report.
    pub branch: String,
    /// The report generated when running `cargo llvm-cov --json`
    pub json_report: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReportHistory {
    /// Url of the git repository associated to this report.
    ///
    /// We need to clone the repository so we can package the sources files into the export.
    pub repository_name: String,
    /// Branch of git repository associated to this report.
    pub branch: String,
    pub name: String,
    pub comparison: Comparison,
    pub date: NaiveDateTime,
}

impl Request {
    /// try to extract the project name from the git path, or return an url safe version of the git address
    pub fn repository_name(&self) -> String {
        let capture = config::REPOSITORY_REGEX.captures(&self.git);
        if let Some(name) = capture.map(|c| c.name("name").unwrap().as_str()) {
            name.replace('/', "-").to_lowercase()
        } else {
            utils::url_safe_string(&self.git)
        }
    }

    /// try to extract the project name from the git path
    pub fn raw_repository_name(&self) -> String {
        let capture = config::REPOSITORY_REGEX.captures(&self.git);
        if let Some(name) = capture.map(|c| c.name("name").unwrap().as_str()) {
            name.to_string()
        } else {
            utils::url_safe_string(&self.git)
        }
    }

    /// Based on the git url and the branch name
    pub fn unique_name(&self) -> String {
        let mut result = self.repository_name();
        result.push('-');
        result.push_str(&utils::url_safe_string(&self.branch));
        result
    }
}

pub fn find_matching_project_path<'a>(
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

pub fn save_json(
    request: &Request,
    report: &Report,
    local_repository: &Path,
) -> anyhow::Result<String> {
    let fixed_report = raw_report_with_local_repository(request, report, local_repository)?;
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
    file.write_all(fixed_report.as_bytes())?;
    Ok(json_path.to_string_lossy().into_owned())
}
