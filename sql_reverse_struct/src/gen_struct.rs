use crate::common::CustomConfig;
use async_trait::async_trait;
use quicli::prelude::read_file;
use sql_reverse_error::result::Result;
use sql_reverse_template::table::Table;
use std::collections::{HashMap, BTreeMap};
use regex::Regex;
use std::io::{self, Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

fn write_red(text: &str) -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
    writeln!(&mut stdout, "{}" ,text)
}

#[async_trait]
pub trait GenStruct {
    async fn run(&self, filename: &str) -> Result<Vec<Table>> {
        let tables = async_ok!(self.get_tables())?;
        let table_comment_map = async_ok!(self.get_tables_comment())?;
        let fields_type = async_ok!(self.load_custom_fields_type(filename))?;
        let templates = async_ok!(self.gen_templates(tables, table_comment_map, fields_type))?;
        Ok(templates)
    }

    async fn load_custom_fields_type(&self, filename: &str) -> Result<Option<BTreeMap<String, String>>> {
        if filename == "" {
            return Ok(None)
        }
        let s = read_file(filename)?;
        let fields_type:BTreeMap<String, String> = serde_json::from_str(&s)?;
        Ok(Some(fields_type))
    }

    async fn get_tables(&self) -> Result<Vec<String>>;

    async fn get_tables_comment(&self) -> Result<HashMap<String, String>>;

    async fn gen_templates(
        &self,
        tables: Vec<String>,
        table_comment_map: HashMap<String, String>,
        fields_type: Option<BTreeMap<String, String>>,
    ) -> Result<Vec<Table>>;

    async fn get_field_type(&self, field_type: &str, filed_name: &str, field_type_map: &BTreeMap<String, String>) -> Result<String> {
        for (k, v) in field_type_map.iter() {
            let r = Regex::new(k.trim()).unwrap();
            if r.is_match(&field_type) {
                return Ok(v.to_string());
            }
        }
        write_red(&format!("filed_name:{}, {} field type does not match, default type [String] will be used", filed_name, field_type))?;
        Ok(String::from("String"))
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
