use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Branch {
    pub count: i64,
    pub covered: i64,
    pub notcovered: i64,
    pub percent: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Functions {
    pub count: i64,
    pub covered: i64,
    pub percent: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Instantiations {
    pub count: i64,
    pub covered: i64,
    pub percent: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Lines {
    pub count: i64,
    pub covered: i64,
    pub percent: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Regions {
    pub count: i64,
    pub covered: i64,
    pub notcovered: i64,
    pub percent: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Summary {
    pub branches: Branch,
    pub functions: Functions,
    pub instantiations: Instantiations,
    pub lines: Lines,
    pub regions: Regions,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct File {
    pub filename: String,
    pub summary: Summary,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Function {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CargoLlvmCov {
    pub manifest_path: String,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Data {
    pub files: Vec<File>,
    pub functions: Vec<Function>,
    pub totals: Summary,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Report {
    pub cargo_llvm_cov: CargoLlvmCov,
    pub data: Vec<Data>,
    #[serde(rename = "type")]
    pub kind: String,
    pub version: String,
}
