use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Deserialize, Serialize, Clone)]
pub struct CustomConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub schemaname: Option<String>,
    pub include_tables: Option<Vec<String>>,
    pub exclude_tables: Option<Vec<String>>,
    pub output_dir: String,
}
