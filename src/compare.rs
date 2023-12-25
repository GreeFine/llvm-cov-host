use std::{
    collections::HashMap,
    sync::{LazyLock, RwLock},
};

use serde::Serialize;

use crate::model::Report;

pub static DEFAULT_BRANCH_REPORTS: LazyLock<RwLock<HashMap<String, Report>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

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
