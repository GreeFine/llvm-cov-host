use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{config, model::Report, storage::TypedDb};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Comparison {
    base: Option<f64>,
    new: f64,
    diff: Option<f64>,
}

impl Display for Comparison {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(diff) = self.diff {
            write!(f, "{:.1}%, difference: {:+.1}%", self.new, diff)
        } else {
            write!(f, "{:.1}%", self.new)
        }
    }
}

fn float_precision_two(value: f64) -> f64 {
    (value * 100f64).round() / 100f64
}

fn function_coverage(base: Option<Report>, new: &Report) -> Comparison {
    Comparison {
        base: base
            .as_ref()
            .map(|base_report| float_precision_two(base_report.data[0].totals.functions.percent)),
        new: float_precision_two(new.data[0].totals.functions.percent),
        diff: base.map(|base_report| {
            float_precision_two(
                new.data[0].totals.functions.percent - base_report.data[0].totals.functions.percent,
            )
        }),
    }
}

pub fn default_branch(
    storage: &TypedDb,
    report: &Report,
    branch: &str,
) -> anyhow::Result<Comparison> {
    let base_report = storage.get::<Report>(branch)?;
    let result = function_coverage(base_report, report);
    if branch == config::DEFAULT_REPORT_BRANCH {
        storage.insert(branch, report)?;
    }
    Ok(result)
}

#[test]
fn test_float_precision_two() {
    assert_eq!(float_precision_two(2.12414), 2.12)
}
