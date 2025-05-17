use crate::error::Result;
use crate::reverse_struct::common::CustomConfig;
use crate::reverse_struct::gen_struct::GenStruct;
use crate::table::{Field, Table, Table2Comment};
use crate::template::kit::Kit;
use inflector::Inflector;
use regex::Regex;
use sqlx::Row;
use std::collections::BTreeMap;
use std::sync::{LazyLock, RwLock};

pub static FIELD_TYPE: LazyLock<RwLock<BTreeMap<String, String>>> = LazyLock::new(|| {
    let mut map = BTreeMap::new();
    map.insert(r"^smallint$".to_string(), "i16".to_string());
    map.insert(r"^integer$".to_string(), "i32".to_string());
    map.insert(r"^bigint$".to_string(), "i64".to_string());
    map.insert(r"^decimal$".to_string(), "Decimal".to_string());
    map.insert(r"^numeric$".to_string(), "Decimal".to_string());
    map.insert(r"^real$".to_string(), "Decimal".to_string());
    map.insert(r"^double$".to_string(), "Decimal".to_string());
    map.insert(r"^precision$".to_string(), "Decimal".to_string());
    map.insert(r"^smallserial$".to_string(), "u16".to_string());
    map.insert(r"^serial$".to_string(), "u32".to_string());
    map.insert(r"^bigserial$".to_string(), "u64".to_string());
    map.insert(r"^money$".to_string(), "Decimal".to_string());
    map.insert(r"^char$".to_string(), "String".to_string());
    map.insert(r"^char\(\d+\)$".to_string(), "String".to_string());
    map.insert(r"^varchar$".to_string(), "String".to_string());
    map.insert(r"^varchar\(\d+\)$".to_string(), "String".to_string());
    map.insert(r"^text$".to_string(), "String".to_string());
    map.insert(r"^bytea$".to_string(), "Vec<u8>".to_string());
    map.insert(r"^timestamp$".to_string(), "NaiveDateTime".to_string());
    map.insert(
        r"^timestamp with time zone$".to_string(),
        "NaiveDateTime".to_string(),
    );
    map.insert(
        r"^time with time zone$".to_string(),
        "NaiveDateTime".to_string(),
    );
    map.insert(r"^time$".to_string(), "NaiveDateTime".to_string());
    map.insert(r"^date$".to_string(), "Date".to_string());
    map.insert(r"^interval$".to_string(), "String".to_string());
    map.insert(r"^uuid$".to_string(), "String".to_string());
    map.insert(r"^xml$".to_string(), "String".to_string());
    map.insert(r"^json$".to_string(), "String".to_string());
    map.insert(r"^jsonb$".to_string(), "String".to_string());
    map.insert(r"^jsonpath$".to_string(), "String".to_string());
    RwLock::new(map)
});

#[derive(Debug)]
pub struct PostgresStruct {
    pub config: CustomConfig,
    pub pool: sqlx::PgPool,
}

impl Kit for PostgresStruct {}

impl PostgresStruct {
    pub async fn init(config: CustomConfig) -> Result<Self> {
        let pool = sqlx::PgPool::connect(&config.db_url).await?;
        Ok(Self { config, pool })
    }
}

const TABLES_SQL: &str = "SELECT
    c.relname as table_name,
    CAST(obj_description(c.oid, 'pg_class') AS VARCHAR) as table_comment
FROM pg_class c
WHERE c.relkind = 'r'
  AND c.relnamespace = (SELECT oid FROM pg_namespace WHERE nspname = $1)
  AND c.relname NOT LIKE 'pg_%'
  AND c.relname NOT LIKE 'sql_%'";

const TABLE_FIELDS: &str = "select a.attname                             as field_name,
       format_type(a.atttypid, a.atttypmod)  as field_type,
       a.attnotnull                          as is_null,
       col_description(a.attrelid, a.attnum) as comment,
       pg_get_expr(d.adbin, d.adrelid)       as default_value
from pg_class c,
     pg_attribute a
         left join (select a.attname, ad.adbin, ad.adrelid
                    from pg_class c,
                         pg_attribute a,
                         pg_attrdef ad
                    where relname = $1
                      and ad.adrelid = c.oid
                      and adnum = a.attnum
                      and attrelid = c.oid) as d on a.attname = d.attname
where c.relname = $2
  and a.attrelid = c.oid
  and a.attnum > 0";

const INDEX_SQL: &str = "SELECT indexdef FROM pg_indexes WHERE schemaname = $1 and tablename = $2";
impl GenStruct for PostgresStruct {
    async fn get_tables(&self) -> Result<Vec<Table2Comment>> {
        let mut pool = self.pool.acquire().await?;
        let mut tables = sqlx::query_as::<_, Table2Comment>(TABLES_SQL)
            .bind(self.config.schemaname.to_owned().unwrap_or_default())
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
        for table in tables {
            let mut pool = self.pool.acquire().await?;
            let fields = sqlx::query_as::<_, Field>(TABLE_FIELDS)
                .bind(&table.table_name)
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
        let rows = sqlx::query(INDEX_SQL)
            .bind(self.config.schemaname.to_owned().unwrap_or_default())
            .bind(table_name)
            .fetch_all(&self.pool)
            .await?;
        let mut index_list = vec![];
        let mut unique_list = vec![];
        let re = Regex::new("\\(.*\\)")?;
        for row in rows {
            let value: &str = row.get(0);
            if re.is_match(value) {
                let data = re.find(value).unwrap();
                let v: &[u8] = value
                    .as_bytes()
                    .get(data.start() + 1..data.end() - 1)
                    .unwrap();
                let v = String::from_utf8_lossy(v).to_string();
                let v: Vec<&str> = v.split(", ").collect();
                let mut index = vec![];
                for v in v {
                    index.push(v.to_owned());
                }
                if value.contains("UNIQUE") || value.contains("unique") {
                    unique_list.push(index)
                } else {
                    index_list.push(index);
                }
            }
        }
        Ok((index_list, unique_list))
    }
}
