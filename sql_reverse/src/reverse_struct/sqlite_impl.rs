use crate::error::Result;
use crate::reverse_struct::common::CustomConfig;
use crate::reverse_struct::gen_struct::GenStruct;
use crate::table::sqlite::Fields;
use crate::table::{Table, Table2Comment};
use crate::template::kit::Kit;
use fn_macro::btreemap;
use inflector::Inflector;
use regex::Regex;
use sqlx::Row;
use std::collections::BTreeMap;
use std::sync::{LazyLock, RwLock};

pub static FIELD_TYPE: LazyLock<RwLock<BTreeMap<String, String>>> = LazyLock::new(|| {
    RwLock::new(btreemap!(
        r"^INTEGER$".to_string() => "i64".to_string(),
        r"^integer$".to_string() => "i64".to_string(),
        r"^TEXT$".to_string() => "String".to_string(),
        r"^BLOB$".to_string() => "Vec<u8>".to_string(),
        r"^ANY$".to_string() => "serde_json::Value".to_string(),
        r"^REAL$".to_string() => "f64".to_string(),
        r"^INT$".to_string() => "i32".to_string(),
        r"^bool$".to_string() => "bool".to_string(),
        r"^BOOLEAN$".to_string() => "bool".to_string(),
        r"^VARCHAR".to_string() => "String".to_string(),
        r"^TIMESTAMP".to_string() => "chrono::NaiveDateTime".to_string(),

    ))
});
pub struct SqliteImpl {
    pub config: CustomConfig,
    pub pool: sqlx::SqlitePool,
}
impl Kit for SqliteImpl {}
impl SqliteImpl {
    pub async fn init(config: CustomConfig) -> Result<Self> {
        let pool = sqlx::SqlitePool::connect(&config.db_url).await?;
        Ok(SqliteImpl { config, pool })
    }
}

const TABLES_SQL: &str = "select name as table_name from sqlite_master where type='table'";
const FIELD_SQL: &str = "select sql from sqlite_master where type='table' and name = ?";

const INDEX_SQL: &str = "select sql from sqlite_master where type='index' and name = ?";
impl GenStruct for SqliteImpl {
    async fn get_tables(&self) -> Result<Vec<Table2Comment>> {
        let mut pool = self.pool.acquire().await?;
        let mut tables = sqlx::query_as::<_, Table2Comment>(TABLES_SQL)
            .fetch_all(&mut *pool)
            .await?;
        self.filter_tables(
            &mut tables,
            &self.config.include_tables,
            &self.config.exclude_tables,
        )
        .await;
        Ok(tables)
    }

    async fn update_type_fields(&self, map: Option<BTreeMap<String, String>>) {
        if let Some(map) = map {
            *FIELD_TYPE.write().unwrap() = map;
        }
    }

    async fn gen_templates(&self, tables: Vec<Table2Comment>) -> Result<Vec<Table>> {
        let mut templates = vec![];
        let mut pool = self.pool.acquire().await?;
        for table in tables {
            let fields = sqlx::query_as::<_, Fields>(FIELD_SQL)
                .bind(&table.table_name)
                .fetch_one(&mut *pool)
                .await?;
            let mut struct_name = table.table_name.clone().to_camel_case();
            struct_name = Self::first_char_to_uppercase(&struct_name);
            let mut table = Table {
                table_name: table.table_name.to_owned(),
                struct_name,
                fields: fields.fields,
                comment: table.table_comment.unwrap_or_default(),
                index_key: vec![],
                unique_key: vec![],
            };
            if !fields.keys.is_empty() {
                table.unique_key.push(fields.keys);
            }
            let (index_key, unique_key) = self.index_key(&table.table_name).await?;
            table.index_key = index_key;
            table.unique_key.extend(unique_key);
            templates.push(table);
        }
        Ok(templates)
    }

    async fn index_key(&self, table_name: &str) -> Result<(Vec<Vec<String>>, Vec<Vec<String>>)> {
        let rows = sqlx::query(INDEX_SQL)
            .bind(table_name)
            .fetch_all(&self.pool)
            .await?;
        let mut index_list = vec![];
        let mut unique_list = vec![];
        let re = Regex::new("\\(.*\\)")?;
        for row in rows {
            let sql: &str = row.get(0);
            if re.is_match(sql) {
                if let Some(data) = re.find(sql) {
                    let index: Vec<String> =
                        data.as_str().split(",").map(|a| a.to_string()).collect();
                    if sql.contains("unique") {
                        unique_list.push(index);
                    } else {
                        index_list.push(index);
                    }
                }
            }
        }
        Ok((index_list, unique_list))
    }
}
