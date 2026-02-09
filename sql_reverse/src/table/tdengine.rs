use serde::Deserialize;

/// Row type for information_schema.ins_stables query results
#[derive(Debug, Clone, Deserialize)]
pub struct TdengineStable {
    pub stable_name: String,
    pub table_comment: Option<String>,
}

/// Row type for information_schema.ins_tables query results
#[derive(Debug, Clone, Deserialize)]
pub struct TdengineNormalTable {
    pub table_name: String,
    pub table_comment: Option<String>,
}

/// Row type for DESCRIBE command results
#[derive(Debug, Clone, Deserialize)]
pub struct TdengineDescribeRow {
    pub field: String,
    #[serde(rename = "type")]
    pub field_type: String,
    #[allow(dead_code)]
    pub length: i32,
    /// "TAG" for tag columns, empty string for regular columns
    pub note: String,
}
