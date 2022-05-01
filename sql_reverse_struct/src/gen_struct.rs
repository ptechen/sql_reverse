use async_trait::async_trait;
use sql_reverse_error::result::Result;
use sql_reverse_template::table::Table;
use crate::common::CustomConfig;
use quicli::prelude::read_file;

#[async_trait]
pub trait GenStruct {
    async fn run(&self) -> Result<Vec<Table>>;

    async fn get_rust_type(&self, field_type: &str) -> Result<String>;

    /// 字符串首字母大写
    async fn first_char_to_uppercase(&self, params: &str) -> Result<String> {
        let mut v: Vec<char> = params.chars().collect();
        v[0] = v[0].to_uppercase().nth(0).unwrap();
        let res = v.into_iter().collect();
        Ok(res)
    }

    async fn load(filename: &str) -> Result<CustomConfig> {
        let s = read_file(filename)?;
        let config: CustomConfig = serde_yaml::from_str(&s)?;
        Ok(config)
    }

    async fn filter_tables(
        &self,
        tables: Vec<String>,
        exclude_tables: Vec<String>,
    ) -> Result<Vec<String>> {
        let mut cur_tables = vec![];
        for table in tables.iter() {
            if !exclude_tables.contains(table) {
                cur_tables.push(table.to_owned());
            }
        }
        Ok(cur_tables)
    }
}
