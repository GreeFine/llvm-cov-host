use serde::Serialize;

use crate::model::Report;

#[derive(Debug, Default, Serialize)]
pub struct Comparison {
    base: f64,
    new: f64,
    diff: f64,
}
pub fn function_coverage<'a>(base: &'a Report, new: &'a Report) -> Comparison {
    Comparison {
        base: base.data[0].totals.functions.percent,
        new: new.data[0].totals.functions.percent,
        diff: base.data[0].totals.functions.percent - new.data[0].totals.functions.percent,
    }
}
