use async_trait::async_trait;
use inflector::Inflector;
use mysql::prelude::*;
use mysql::Row;
use mysql::*;
use once_cell::sync::Lazy;
use quicli::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sql_reveser_error::result::Result;
use sql_reveser_template::gen_struct::GenStruct;
use sql_reveser_template::table::{Field, Table};
use std::collections::HashMap;

static FIELD_TYPE: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert(r"^int$", "i32");
    map.insert(r"^int unsigned$", "u32");
    map.insert(r"^int\(\d+\)$", "i32");
    map.insert(r"^int\(\d+\) unsigned$", "u32");
    map.insert(r"^integer$", "i32");
    map.insert(r"^integer unsigned$", "u32");
    map.insert(r"^integer\(\d+\)$", "i32");
    map.insert(r"^integer\(\d+\) unsigned$", "u32");
    map.insert(r"^tinyint$", "i8");
    map.insert(r"^tinyint unsigned$", "u8");
    map.insert(r"^tinyint\(\d+\)$", "i8");
    map.insert(r"^tinyint\(\d+\) unsigned$", "u8");
    map.insert(r"^smallint$", "i16");
    map.insert(r"^smallint unsigned$", "u16");
    map.insert(r"^smallint\(\d+\)$", "i16");
    map.insert(r"^smallint\(\d+\) unsigned$", "u16");
    map.insert(r"^mediumint$", "i32");
    map.insert(r"^mediumint unsigned$", "u32");
    map.insert(r"^mediumint\(\d+\)$", "i32");
    map.insert(r"^mediumint\(\d+\) unsigned$", "u32");
    map.insert(r"^bigint$", "i64");
    map.insert(r"^bigint unsigned$", "u64");
    map.insert(r"^bigint\(\d+\)$", "i64");
    map.insert(r"^bigint\(\d+\) unsigned$", "u64");
    map.insert(r"^float", "f32");
    map.insert(r"^double", "f64");
    map.insert(r"^decimal", "Decimal");
    map.insert(r"^date$", "Date");
    map.insert(r"^datetime$", "NaiveDateTime");
    map.insert(r"^timestamp$", "NaiveDateTime");
    map.insert(r"year", "Year");
    map.insert(r"char", "String");
    map.insert(r"text", "String");
    map.insert(r"blob", "Vec<u8>");
    map
});

#[derive(Default, Debug, Deserialize, Serialize, Clone)]
pub struct MysqlStruct {
    pub config: CustomConfig,
}

#[derive(Default, Debug, Deserialize, Serialize, Clone)]
pub struct CustomConfig {
    pub host: String,
    pub port: String,
    pub username: String,
    pub password: String,
    pub database: String,
    pub include_tables: Option<Vec<String>>,
    pub exclude_tables: Option<Vec<String>>,
    pub output_dir: String,
}

pub struct GenTemplateData {
    pub table_name: String,
    pub struct_name: String,
    pub mysql_rows: Option<Vec<Row>>,
    pub table_comment: String,
}

impl MysqlStruct {
    pub fn new(config: CustomConfig) -> Result<MysqlStruct> {
        Ok(MysqlStruct { config })
    }

    pub fn load(filename: &str) -> Result<MysqlStruct> {
        let s = read_file(filename)?;
        let config: CustomConfig = serde_yaml::from_str(&s)?;
        MysqlStruct::new(config)
    }

    async fn gen_template_data(&self, gen_template_data: GenTemplateData) -> Result<Table> {
        let mut fields = vec![];
        let mysql_rows = gen_template_data.mysql_rows.unwrap();
        for row in mysql_rows.iter() {
            let field_name: String = row.get(0).unwrap();
            let mut field_type: String = row.get(1).unwrap();
            field_type = self.get_rust_type(&field_type).await?;
            let is_null: String = row.get(3).unwrap_or_default();
            let mut cur_is_null = 0;
            if is_null == "YES" {
                cur_is_null = 1;
            }
            let comment: String = row.get(8).unwrap_or(String::new());
            let field = Field {
                field_name,
                field_type,
                comment,
                is_null: cur_is_null,
            };
            fields.push(field);
        }
        let temp = Table::new(
            gen_template_data.table_name,
            gen_template_data.struct_name,
            fields,
            gen_template_data.table_comment,
        );
        Ok(temp)
    }
}

#[async_trait]
impl GenStruct for MysqlStruct {
    async fn run(&self) -> Result<Vec<Table>> {
        let url = format!(
            "mysql://{}@{}{}:{}/{}",
            self.config.username,
            self.config.password,
            self.config.host,
            self.config.password,
            self.config.database
        );
        let opts = Opts::from_url(&url)?;
        let pool = Pool::new(opts)?;
        let mut conn = pool.get_conn()?;
        let tables;
        let include_tables = self.config.include_tables.as_ref();
        if include_tables.is_some() {
            tables = include_tables.unwrap().to_owned();
        } else {
            let sql = format!(
                "select table_name from information_schema.tables where table_schema= '{}'",
                self.config.database
            );
            tables = conn.query(sql)?;
        }
        let tables_status: Vec<Row> = conn.query("show table status").unwrap();
        let mut table_comment_map = HashMap::new();
        for row in tables_status.iter() {
            let table_name: String = row.get(0).unwrap();
            let table_comment: Option<String> = row.get(17);
            if table_comment.is_some() {
                let table_comment = table_comment.unwrap();
                if table_comment != "" {
                    table_comment_map.insert(table_name, table_comment);
                };
            };
        }
        let mut exclude_tables: Vec<String> = vec![];
        if self.config.exclude_tables.is_some() {
            exclude_tables = self.config.exclude_tables.as_ref().unwrap().to_owned();
        };
        let mut templates = vec![];
        for table_name in tables.iter() {
            if exclude_tables.contains(table_name) {
                continue;
            };
            let sql = format!("show full columns from {}", table_name);
            let mysql_rows: Vec<Row> = conn.query(&sql)?;
            let mut struct_name = table_name.to_camel_case();
            struct_name = self.first_char_to_uppercase(&struct_name).await?;
            let default = &String::new();
            let table_comment = table_comment_map
                .get(table_name)
                .unwrap_or(default)
                .to_string();
            let mysql_rows = Some(mysql_rows);
            let gen_template_data = GenTemplateData {
                table_name: table_name.to_owned(),
                struct_name,
                mysql_rows,
                table_comment,
            };
            let template = self.gen_template_data(gen_template_data).await?;
            templates.push(template);
        }
        Ok(templates)
    }

    async fn get_rust_type(&self, field_type: &str) -> Result<String> {
        for (k, v) in FIELD_TYPE.iter() {
            let r = Regex::new(k.trim()).unwrap();
            if r.is_match(&field_type) {
                return Ok(v.to_string());
            }
        }
        Ok(String::new())
    }
}
