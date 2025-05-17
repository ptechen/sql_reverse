use crate::error::Result;
use crate::reverse_struct::common::CustomConfig;
use crate::reverse_struct::gen_struct::GenStruct;
use crate::table::{Field, Table, Table2Comment};
use crate::template::kit::Kit;
use fn_macro::{btreemap, if_else};
use inflector::Inflector;
use sqlx::Row;
use std::collections::BTreeMap;
use std::sync::{LazyLock, RwLock};

pub static FIELD_TYPE: LazyLock<RwLock<BTreeMap<String, String>>> = LazyLock::new(|| {
    let map = btreemap!(
        "^varbinary\\(\\d+\\)$".to_string() => "Vec<u8>".to_string(),
        "^binary\\(\\d+\\)$".to_string() => "Vec<u8>".to_string(),
        "^bigint unsigned$".to_string() => "u64".to_string(),
        "^bigint$".to_string() => "i64".to_string(),
        "^bigint\\(\\d+\\) unsigned$".to_string() => "u64".to_string(),
        "^bigint\\(\\d+\\)$".to_string() => "i64".to_string(),
        "^date$".to_string() => "Date".to_string(),
        "^datetime$".to_string() => "chrono::NaiveDateTime".to_string(),
        "^decimal".to_string() => "sqlx::types::Decimal".to_string(),
        "^double".to_string() => "f64".to_string(),
        "^float".to_string() => "f32".to_string(),
        "^int unsigned$".to_string() => "u32".to_string(),
        "^int$".to_string() => "i32".to_string(),
        "^int\\(\\d+\\) unsigned$".to_string() => "u32".to_string(),
        "^int\\(\\d+\\)$".to_string() => "i32".to_string(),
        "^integer unsigned$".to_string() => "u32".to_string(),
        "^integer$".to_string() => "i32".to_string(),
        "^integer\\(\\d+\\) unsigned$".to_string() => "u32".to_string(),
        "^integer\\(\\d+\\)$".to_string() => "i32".to_string(),
        "^json$".to_string() => "serde_json:: =>Value".to_string(),
        "^mediumint unsigned$".to_string() => "u32".to_string(),
        "^mediumint$".to_string() => "i32".to_string(),
        "^mediumint\\(\\d+\\) unsigned$".to_string() => "u32".to_string(),
        "^mediumint\\(\\d+\\)$".to_string() => "i32".to_string(),
        "^smallint unsigned$".to_string() => "u16".to_string(),
        "^smallint$".to_string() => "i16".to_string(),
        "^smallint\\(\\d+\\) unsigned$".to_string() => "u16".to_string(),
        "^smallint\\(\\d+\\)$".to_string() => "i16".to_string(),
        "^timestamp$".to_string() => "chrono::NaiveDateTime".to_string(),
        "^tinyint unsigned$".to_string() => "u8".to_string(),
        "^tinyint$".to_string() => "i8".to_string(),
        "^tinyint\\(\\d+\\) unsigned$".to_string() => "u8".to_string(),
        "^tinyint\\(1\\)$".to_string() => "bool".to_string(),
        "^tinyint\\(\\d+\\)$".to_string() => "i8".to_string(),
        "^bit\\(1\\)$".to_string() => "bool".to_string(),
        "^bit$".to_string() => "bool".to_string(),
        "blob".to_string() => "Vec<u8>".to_string(),
        "char".to_string() => "String".to_string(),
        "text".to_string() => "String".to_string(),
        "year".to_string() => "Year".to_string()
    );
    RwLock::new(map)
});

#[derive(Debug, Clone)]
pub struct MysqlStruct {
    pub config: CustomConfig,
    pub pool: sqlx::MySqlPool,
}

impl Kit for MysqlStruct {}

impl MysqlStruct {
    pub async fn init(config: CustomConfig) -> Result<Self> {
        let pool = sqlx::MySqlPool::connect(&config.db_url).await?;
        Ok(Self { config, pool })
    }
}
const FIELD_SQL:&str = "SELECT CAST(COLUMN_NAME as CHAR ) as field_name, CAST(DATA_TYPE as CHAR ) as field_type, case when CAST(IS_NULLABLE as CHAR) = 'NO' THEN 0 else 1 END as is_null,
       CAST(COLUMN_COMMENT as CHAR ) as comment, CAST(COLUMN_DEFAULT as CHAR ) as default_value
FROM INFORMATION_SCHEMA.COLUMNS
WHERE table_schema = DATABASE() AND table_name = ?";
const TABLES_SQL: &str = "SELECT CAST(TABLE_NAME AS CHAR) as table_name, CAST(TABLE_COMMENT as CHAR) as table_comment FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_SCHEMA = DATABASE()";

impl GenStruct for MysqlStruct {
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
            let fields = sqlx::query_as::<_, Field>(FIELD_SQL)
                .bind(&table.table_name)
                .fetch_all(&mut *pool)
                .await?;
            let mut struct_name = table.table_name.clone().to_camel_case();
            struct_name = Self::first_char_to_uppercase(&struct_name);
            let mut table = Table {
                table_name: table.table_name.to_owned(),
                struct_name,
                fields,
                comment: table.table_comment.unwrap_or_default(),
                index_key: vec![],
                unique_key: vec![],
            };
            let (index_key, unique_key) = self.index_key(&table.table_name).await?;
            table.index_key = index_key;
            table.unique_key = unique_key;
            templates.push(table);
        }
        Ok(templates)
    }

    async fn index_key(&self, table_name: &str) -> Result<(Vec<Vec<String>>, Vec<Vec<String>>)> {
        let indexes = sqlx::query(&format!("show index from {}", table_name))
            .fetch_all(&self.pool)
            .await?;
        let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut unique_map: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for index in indexes {
            let key = String::from_utf8(index.get(2))?;
            let i: i8 = index.get(1);
            let is_unique = if_else!(i == 1, 0, 1);
            let field_name = String::from_utf8(index.get(4))?;
            if is_unique == 0 {
                let v = unique_map.get(&key);
                if v.is_none() {
                    unique_map.insert(key, vec![field_name]);
                } else {
                    let mut v = v.unwrap().to_owned();
                    v.push(field_name);
                    unique_map.insert(key, v);
                }
            } else {
                let v = map.get(&key);
                if v.is_none() {
                    map.insert(key, vec![field_name]);
                } else {
                    let mut v = v.unwrap().to_owned();
                    v.push(field_name);
                    map.insert(key, v);
                }
            }
        }
        let mut list = vec![];
        for (_, val) in map {
            list.push(val);
        }
        let mut unique_list = vec![];
        for (_, val) in unique_map {
            unique_list.push(val);
        }
        Ok((list, unique_list))
    }
}
