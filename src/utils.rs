use std::{fs, path::Path};

use crate::config;

pub fn init_environment() {
    dotenvy::dotenv().ok();
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();

    let json_dir = Path::new(config::JSON_REPORTS_DIR);
    if !json_dir.is_dir() {
        fs::create_dir_all(json_dir).unwrap();
    };
    let html_dir = Path::new(config::HTML_REPORTS_DIR);
    if !html_dir.is_dir() {
        fs::create_dir_all(html_dir).unwrap();
    };
    let repos_dir = Path::new(config::REPOSITORIES_DIR);
    if !repos_dir.is_dir() {
        fs::create_dir_all(repos_dir).unwrap();
    };
}

pub fn url_safe_string(input: &str) -> String {
    let mut prev_is_dash = false;
    input
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
        .collect()
}
