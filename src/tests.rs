// use actix_web::{http::header::ContentType, test, App};

use crate::git;

#[actix_web::test]
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
    let path = git::pull_or_clone("https://github.com/GreeFine/llvm-cov-host.git", "main").unwrap();
    assert!(path.exists())
}
