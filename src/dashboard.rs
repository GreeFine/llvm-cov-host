use askama::Template;

use crate::report::ReportHistory;

#[derive(Template)]
#[template(path = "dashboard.jinja")]

pub struct DashBoardTemplate {
    pub reports: Vec<ReportHistory>,
}
