use crate::model::Report;

pub fn function_coverage<'a>(base: &'a Report, diff: &'a Report) -> f64 {
    base.data[0].totals.functions.percent - diff.data[0].totals.functions.percent
}
