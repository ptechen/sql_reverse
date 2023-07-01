use crate::common::CustomConfig;
use crate::gen_struct::GenStruct;
use async_trait::async_trait;
use inflector::Inflector;
use mysql::prelude::*;
use mysql::Row;
use mysql::*;
use once_cell::sync::Lazy;
use sql_reverse_error::result::Result;
use sql_reverse_template::table::{Field, Table};
use std::collections::{HashMap, BTreeMap};

pub static FIELD_TYPE: Lazy<BTreeMap<String, String>> = Lazy::new(|| {
    let mut map = BTreeMap::new();
    map.insert(r"^int$".to_string(), "i32".to_string());
    map.insert(r"^int unsigned$".to_string(), "u32".to_string());
    map.insert(r"^int\(\d+\)$".to_string(), "i32".to_string());
    map.insert(r"^int\(\d+\) unsigned$".to_string(), "u32".to_string());
    map.insert(r"^integer$".to_string(), "i32".to_string());
    map.insert(r"^integer unsigned$".to_string(), "u32".to_string());
    map.insert(r"^integer\(\d+\)$".to_string(), "i32".to_string());
    map.insert(r"^integer\(\d+\) unsigned$".to_string(), "u32".to_string());
    map.insert(r"^tinyint$".to_string(), "i8".to_string());
    map.insert(r"^tinyint unsigned$".to_string(), "u8".to_string());
    map.insert(r"^tinyint\(\d+\)$".to_string(), "i8".to_string());
    map.insert(r"^tinyint\(\d+\) unsigned$".to_string(), "u8".to_string());
    map.insert(r"^smallint$".to_string(), "i16".to_string());
    map.insert(r"^smallint unsigned$".to_string(), "u16".to_string());
    map.insert(r"^smallint\(\d+\)$".to_string(), "i16".to_string());
    map.insert(r"^smallint\(\d+\) unsigned$".to_string(), "u16".to_string());
    map.insert(r"^mediumint$".to_string(), "i32".to_string());
    map.insert(r"^mediumint unsigned$".to_string(), "u32".to_string());
    map.insert(r"^mediumint\(\d+\)$".to_string(), "i32".to_string());
    map.insert(r"^mediumint\(\d+\) unsigned$".to_string(), "u32".to_string());
    map.insert(r"^bigint$".to_string(), "i64".to_string());
    map.insert(r"^bigint unsigned$".to_string(), "u64".to_string());
    map.insert(r"^bigint\(\d+\)$".to_string(), "i64".to_string());
    map.insert(r"^bigint\(\d+\) unsigned$".to_string(), "u64".to_string());
    map.insert(r"^float".to_string(), "f32".to_string());
    map.insert(r"^double".to_string(), "f64".to_string());
    map.insert(r"^decimal".to_string(), "Decimal".to_string());
    map.insert(r"^date$".to_string(), "Date".to_string());
    map.insert(r"^datetime$".to_string(), "NaiveDateTime".to_string());
    map.insert(r"^timestamp$".to_string(), "NaiveDateTime".to_string());
    map.insert(r"year".to_string(), "Year".to_string());
    map.insert(r"char".to_string(), "String".to_string());
    map.insert(r"text".to_string(), "String".to_string());
    map.insert(r"blob".to_string(), "Vec<u8>".to_string());
    map.insert(r"^json$".to_string(), "String".to_string());
    map
});

#[derive(Debug, Clone)]
pub struct MysqlStruct {
    pub config: CustomConfig,
    pub pool: Pool,
}

pub struct GenTemplateData {
    pub table_name: String,
    pub struct_name: String,
    pub sql_rows: Vec<Row>,
    pub table_comment: String,
}

impl MysqlStruct {
    pub fn new(config: CustomConfig) -> Result<Self> {
        let url = format!(
            "mysql://{}:{}@{}:{}/{}",
            config.username, config.password, config.host, config.port, config.database
        );
        let opts = Opts::from_url(&url)?;
        let pool = Pool::new(opts)?;
        Ok(Self { config, pool })
    }

    async fn gen_template_data(&self, gen_template_data: GenTemplateData, fields_type: &Option<BTreeMap<String, String>>) -> Result<Table> {
        let default_type = FIELD_TYPE.clone();
        let fields_type = fields_type.as_ref().unwrap_or(&default_type);
        let mut fields = vec![];
        for row in gen_template_data.sql_rows.iter() {
            let field_name: String = row.get(0).unwrap();
            let camel_name: String = field_name.to_camel_case();
            let capitalized_camel_case = async_ok!(self.first_char_to_uppercase(&camel_name))?;
            let database_field_type: String = row.get(1).unwrap();
            let field_type = async_ok!(self.get_field_type(&database_field_type, &field_name, fields_type))?;
            let is_null: String = row.get(3).unwrap_or_default();
            let mut cur_is_null = 0;
            if is_null == "YES" {
                cur_is_null = 1;
            }
            let default_value: Option<String> = row.get(5).unwrap_or(None);
            let comment: String = row.get(8).unwrap_or(String::new());
            let field = Field {
                field_name,
                FieldName: capitalized_camel_case,
                fieldName: camel_name,
                database_field_type,
                field_type,
                comment,
                is_null: cur_is_null,
                default: default_value
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

    // async fn index_key(&self, conn: &mut PooledConn, table_name: &str) -> Result<Vec<Vec<String>>> {
    //     let indexs: Vec<(String, String)> = conn.query(&format!("select CONSTRAINT_NAME,COLUMN_NAME from INFORMATION_SCHEMA.KEY_COLUMN_USAGE t where t.TABLE_NAME ='{}'", table_name))?;
    //     let mut key = String::new();
    //     let mut list = vec![];
    //     let mut cur = vec![];
    //     for index in indexs {
    //         if key == "" {
    //             key = index.0;
    //             cur.push(index.1)
    //         } else if key == index.0 {
    //             key = index.0;
    //             cur.push(index.1);
    //         } else {
    //             key = index.0;
    //             list.push(cur);
    //             cur = vec![];
    //             cur.push(index.1);
    //         }
    //     }
    //     if cur.len() > 0 {
    //         list.push(cur);
    //     }
    //     Ok(list)
    // }
    async fn index_key(&self, conn: &mut PooledConn, table_name: &str) -> Result<Vec<Vec<String>>> {
        let indexs:Vec<Row>  = conn.query(&format!("show index from {}", table_name))?;
        let mut map:BTreeMap<String, Vec<String>>= BTreeMap::new();
        for index in indexs {
            let key:String = index.get(2).unwrap();
            let field_name:String = index.get(4).unwrap();
            let v = map.get(&key);
            if v.is_none() {
                map.insert(key, vec![field_name]);
            } else {
                let mut v = v.unwrap().to_owned();
                v.push(field_name);
                map.insert(key, v);
            }
        }
        let mut list = vec![];
        for (_, val) in map {
            list.push(val);
        }
        Ok(list)
    }
}

#[async_trait]
impl GenStruct for MysqlStruct {
    async fn get_tables(&self) -> Result<Vec<String>> {
        let mut tables;
        let include_tables = self.config.include_tables.as_ref();
        if include_tables.is_some() {
            tables = include_tables.unwrap().to_owned();
        } else {
            let sql = format!(
                "select table_name from information_schema.tables where table_schema= '{}'",
                self.config.database
            );
            let mut conn = self.pool.get_conn()?;
            tables = conn.query(sql)?;
            let exclude_tables = self.config.exclude_tables.as_ref();
            if exclude_tables.is_some() {
                tables = async_ok!(self.filter_tables(tables, exclude_tables.unwrap().to_owned()))?;
            }
        }
        Ok(tables)
    }

    async fn get_tables_comment(&self) -> Result<HashMap<String, String>> {
        let mut conn = self.pool.get_conn()?;
        let tables_status: Vec<Row> = conn.query("show table status").unwrap();
        let mut table_comment_map = HashMap::new();
        for row in tables_status.iter() {
            let table_name: String = row.get(0).unwrap();
            let table_comment: Option<String> = row.get(17);
            table_comment_map.insert(table_name, table_comment.unwrap_or_default());
        }
        Ok(table_comment_map)
    }

    async fn gen_templates(
        &self,
        tables: Vec<String>,
        table_comment_map: HashMap<String, String>,
        fields_type: Option<BTreeMap<String, String>>,
    ) -> Result<Vec<Table>> {
        let mut templates = vec![];
        let mut conn = self.pool.get_conn()?;
        for table_name in tables.iter() {
            let sql = format!("show full columns from {}", table_name);
            let sql_rows: Vec<Row> = conn.query(&sql)?;
            let mut struct_name = table_name.to_camel_case();
            struct_name = self.first_char_to_uppercase(&struct_name).await?;
            let default = &String::new();
            let table_comment = table_comment_map
                .get(table_name)
                .unwrap_or(default)
                .to_string();
            let gen_template_data = GenTemplateData {
                table_name: table_name.to_owned(),
                struct_name,
                sql_rows,
                table_comment,
            };
            let mut table = self.gen_template_data(gen_template_data, &fields_type).await?;
            table.index_key = self.index_key(&mut conn, table_name).await?;
            templates.push(table);
        }
        Ok(templates)
    }
}
