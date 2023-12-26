use std::sync::LazyLock;

use regex::Regex;

pub const JSON_REPORTS_DIR: &str = "./output/json-reports/";
pub const HTML_REPORTS_DIR: &str = "./output/html-reports/";
pub const REPOSITORIES_DIR: &str = "./output/repositories/";
pub const SLED_DIR: &str = "./output/persistance";

pub static REPOSITORY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^(.*@.*:|https:\/\/.*\.*.\/)(?<name>[A-z-]{0,100}\/[A-z-]{0,100})(\.git)?$"#)
        .unwrap()
});

/// Name of the branch that is used as comparison to calculate the difference in coverage of other branches
pub const DEFAULT_REPORT_BRANCH: &str = "main";
