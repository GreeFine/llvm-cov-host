// use actix_web::{http::header::ContentType, test, App};

use std::fs;

use crate::git;

#[actix_web::test]
#[ignore = "todo"]
async fn test_put_report() {
    todo!()
    // let app = test::init_service(App::new().service(index)).await;
    // let req = test::TestRequest::default()
    //     .insert_header(ContentType::plaintext())
    //     .to_request();
    // let resp = test::call_service(&app, req).await;
    // assert!(resp.status().is_success());
}

#[test]
fn test_git_clone() {
    dotenvy::dotenv().ok();

    let path = git::pull_or_clone("https://github.com/GreeFine/llvm-cov-host.git", "main").unwrap();
    assert!(path.exists());
    let _ = fs::remove_dir_all(&path);
}

#[test]
fn test_git_clone_ssh() {
    dotenvy::dotenv().ok();

    let path = git::pull_or_clone("git@github.com:GreeFine/llvm-cov-host.git", "main").unwrap();
    assert!(path.exists());
    let _ = fs::remove_dir_all(&path);
}
