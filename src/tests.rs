// use actix_web::{http::header::ContentType, test, App};

use std::{fs, path::Path};

use crate::{
    git,
    report::{find_matching_project_path, Request},
};

#[test]
fn test_git_clone_http() {
    dotenvy::dotenv().ok();

    let request = Request {
        branch: "main".to_string(),
        git: "https://github.com/GreeFine/llvm-cov-host.git".to_string(),
        json_report: serde_json::Value::Null,
    };

    let path = git::pull_or_clone(&request).unwrap();
    assert!(path.exists());
    let _ = fs::remove_dir_all(&path);

    let path = git::pull_or_clone(&request).unwrap();
    assert!(path.exists());
    let _ = fs::remove_dir_all(&path);
}

#[test]
fn test_get_names() {
    let request = Request {
        branch: "main/aqwqe/2".to_string(),
        git: "https://github.com/GreeFine/llvm-cov-host".to_string(),
        json_report: serde_json::Value::Null,
    };
    assert_eq!(request.unique_name(), "greefine-llvm-cov-host-main-aqwqe-2");
    assert_eq!(request.repository_name(), "greefine-llvm-cov-host");

    let request = Request {
        branch: "main/aqwqe/2".to_string(),
        git: "weird://github.com/GreeFine/llvm-cov-host".to_string(),
        json_report: serde_json::Value::Null,
    };
    assert_eq!(
        request.unique_name(),
        "weird-github-com-greefine-llvm-cov-host-main-aqwqe-2"
    );
    assert_eq!(
        request.repository_name(),
        "weird-github-com-greefine-llvm-cov-host"
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
