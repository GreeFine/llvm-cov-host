use serde::Serialize;

use crate::{config, model::Report, storage::TypedDb};

#[derive(Debug, Default, Serialize)]
pub struct Comparison {
    base: f64,
    new: f64,
    diff: f64,
}

fn function_coverage<'a>(base: &'a Report, new: &'a Report) -> Comparison {
    Comparison {
        base: base.data[0].totals.functions.percent,
        new: new.data[0].totals.functions.percent,
        diff: base.data[0].totals.functions.percent - new.data[0].totals.functions.percent,
    }
}

pub fn default_branch(
    storage: &TypedDb,
    report: &Report,
    branch: &str,
) -> anyhow::Result<Comparison> {
    let base_comparison = {
        if let Some(default_report) = storage.get::<Report>(branch)? {
            function_coverage(report, &default_report)
        } else {
            Comparison::default()
        }
    };
    if branch == config::DEFAULT_REPORT_BRANCH {
        storage.insert(branch, report)?;
    }
    Ok(base_comparison)
}
