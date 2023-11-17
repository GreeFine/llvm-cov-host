use std::{fs, path::Path};

pub fn init_environment() {
    dotenvy::dotenv().ok();
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();

    let json_dir = Path::new(crate::JSON_REPORTS_DIR);
    if !json_dir.is_dir() {
        fs::create_dir(json_dir).unwrap();
    };
    let html_dir = Path::new(crate::HTML_REPORTS_DIR);
    if !html_dir.is_dir() {
        fs::create_dir(html_dir).unwrap();
    };
}
