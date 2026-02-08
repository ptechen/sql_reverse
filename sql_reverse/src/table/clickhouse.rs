use serde::Deserialize;

/// Row type for system.tables query results
#[derive(Debug, Clone, clickhouse::Row, Deserialize)]
pub struct ClickhouseTable {
    pub name: String,
    pub comment: String,
}

/// Row type for system.columns query results
#[derive(Debug, Clone, clickhouse::Row, Deserialize)]
pub struct ClickhouseField {
    pub name: String,
    pub field_type: String,
    pub comment: String,
    pub default_expression: String,
    /// "DEFAULT", "MATERIALIZED", "ALIAS", or empty
    #[allow(dead_code)]
    pub default_kind: String,
    pub is_in_primary_key: u8,
    pub is_in_sorting_key: u8,
}
