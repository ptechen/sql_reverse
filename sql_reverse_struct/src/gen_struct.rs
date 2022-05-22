use crate::common::CustomConfig;
use async_trait::async_trait;
use quicli::prelude::read_file;
use sql_reverse_error::result::Result;
use sql_reverse_template::table::Table;
use std::collections::HashMap;
use regex::Regex;

#[async_trait]
pub trait GenStruct {
    async fn run(&self) -> Result<Vec<Table>> {
        let tables = async_ok!(self.get_tables())?;
        let table_comment_map = async_ok!(self.get_tables_comment())?;
        let templates = async_ok!(self.gen_templates(tables, table_comment_map))?;
        Ok(templates)
    }

    async fn get_tables(&self) -> Result<Vec<String>>;

    async fn get_tables_comment(&self) -> Result<HashMap<String, String>>;

    async fn gen_templates(
        &self,
        tables: Vec<String>,
        table_comment_map: HashMap<String, String>,
    ) -> Result<Vec<Table>>;

    async fn get_rust_type(&self, field_type: &str, field_type_map: HashMap<&str, &str>) -> Result<String> {
        for (k, v) in field_type_map.iter() {
            let r = Regex::new(k.trim()).unwrap();
            if r.is_match(&field_type) {
                return Ok(v.to_string());
            }
        }
        Ok(String::new())
    }

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
