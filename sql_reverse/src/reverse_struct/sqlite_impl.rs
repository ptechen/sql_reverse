use std::collections::BTreeMap;
use std::sync::{LazyLock, RwLock};
use inflector::Inflector;
use crate::reverse_struct::common::CustomConfig;
use crate::error::result::Result;
use crate::reverse_struct::gen_struct::{GenStruct, Table2Comment};
use crate::table::sqlite::Fields;
use crate::template::kit::Kit;
use crate::table::Table;
pub static FIELD_TYPE: LazyLock<RwLock<BTreeMap<String, String>>> = LazyLock::new(|| {
   RwLock::new(BTreeMap::new()) 
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

const TABLES_SQL:&str = "select name as table_name from sqlite_master where type='table'";
const FIELD_SQL:&str = "select sql from sqlite_master where type='table' and name = ?";

const TABLE_INDEX:&str = "select sql from sqlite_master where type='index' and name = ?";
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
            let (index_key, unique_key) = self.index_key(&table.table_name).await?;
            table.index_key = index_key;
            table.unique_key = unique_key;
            templates.push(table);
        }
        Ok(templates)
    }

    async fn index_key(&self, table_name: &str) -> Result<(Vec<Vec<String>>, Vec<Vec<String>>)> {
        todo!()
    }
}
