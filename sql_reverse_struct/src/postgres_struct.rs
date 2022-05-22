use crate::common::CustomConfig;
use crate::gen_struct::GenStruct;
use async_trait::async_trait;
use inflector::Inflector;
use once_cell::sync::Lazy;
use sql_reverse_error::result::Result;
use sql_reverse_template::table::{Field, Table};
use std::collections::HashMap;
use tokio_postgres::{Client, NoTls, Row};

static FIELD_TYPE: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert(r"^smallint$", "i16");
    map.insert(r"^integer$", "i32");
    map.insert(r"^bigint$", "i64");
    map.insert(r"^decimal$", "Decimal");
    map.insert(r"^numeric$", "Decimal");
    map.insert(r"^real$", "Decimal");
    map.insert(r"^double$", "Decimal");
    map.insert(r"^precision$", "Decimal");
    map.insert(r"^smallserial$", "u16");
    map.insert(r"^serial$", "u32");
    map.insert(r"^bigserial$", "u64");
    map.insert(r"^money$", "Decimal");

    map.insert(r"^char$", "String");
    map.insert(r"^char\(\d+\)$", "String");
    map.insert(r"^varchar$", "String");
    map.insert(r"^varchar\(\d+\)$", "String");
    map.insert(r"^text$", "String");
    map.insert(r"^bytea$", "Vec<u8>");
    map.insert(r"^timestamp$", "NaiveDateTime");
    map.insert(r"^timestamp with time zone$", "NaiveDateTime");
    map.insert(r"^time with time zone$", "NaiveDateTime");
    map.insert(r"^time$", "NaiveDateTime");
    map.insert(r"^date$", "Date");
    map.insert(r"^interval$", "String");
    map.insert(r"^uuid$", "String");
    map.insert(r"^xml$", "String");
    map.insert(r"^json$", "String");
    map.insert(r"^jsonb$", "String");
    map.insert(r"^jsonpath$", "String");
    map
});

#[derive(Debug)]
pub struct PostgresStruct {
    pub config: CustomConfig,
    pub client: Client,
}

pub struct GenTemplateData {
    pub table_name: String,
    pub struct_name: String,
    pub sql_rows: Vec<Row>,
    pub table_comment: String,
}

impl PostgresStruct {
    pub async fn new(config: CustomConfig) -> Result<Self> {
        let (client, connection) = tokio_postgres::Config::new()
            .user(&config.username)
            .password(&config.password)
            .dbname(&config.database)
            .host(&config.host)
            .connect(NoTls)
            .await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                panic!("connection error: {}", e);
            }
        });
        Ok(Self { config, client })
    }

    async fn gen_template_data(&self, gen_template_data: GenTemplateData) -> Result<Table> {
        let mut fields = vec![];
        for row in gen_template_data.sql_rows.iter() {
            let field_name: String = row.get(0);
            if field_name.contains("drop") {
                continue;
            }
            let mut field_type: String = row.get(1);
            field_type = async_ok!(self.get_rust_type(&field_type, FIELD_TYPE.clone()))?;
            let is_null: bool = row.get(2);
            let mut cur_is_null = 0;
            if is_null == false {
                cur_is_null = 1;
            }
            let comment: Option<String> = row.get(3);
            let field = Field {
                field_name,
                field_type,
                comment: comment.unwrap_or_default(),
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
impl GenStruct for PostgresStruct {
    async fn get_tables(&self) -> Result<Vec<String>> {
        let mut tables = vec![];
        let include_tables = self.config.include_tables.as_ref();
        if include_tables.is_some() {
            tables = include_tables.unwrap().to_owned();
        } else {
            let rows = &self
                .client
                .query(
                    &format!(
                        "select tablename from pg_tables where schemaname = '{}'",
                        self.config.schemaname.to_owned().unwrap_or_default()
                    ),
                    &[],
                )
                .await?;
            for row in rows {
                let value: &str = row.get(0);
                tables.push(value.to_string());
            }
            let exclude_tables = self.config.exclude_tables.as_ref();
            if exclude_tables.is_some() {
                tables = async_ok!(self.filter_tables(tables, exclude_tables.unwrap().to_owned()))?;
            }
        }
        Ok(tables)
    }

    async fn get_tables_comment(&self) -> Result<HashMap<String, String>> {
        let rows = &self
            .client
            .query(
                &format!(
                    "select relname,cast(obj_description(relfilenode,'pg_class') as varchar)\
                     as comment from pg_class where relname in \
                     (select tablename from pg_tables where schemaname = '{}')",
                    self.config.schemaname.to_owned().unwrap_or_default()
                ),
                &[],
            )
            .await?;
        let mut table_comment_map = HashMap::new();
        for row in rows {
            let table_name: String = row.get(0);
            let table_comment: Option<String> = row.get(1);
            table_comment_map.insert(table_name, table_comment.unwrap_or_default());
        }
        Ok(table_comment_map)
    }

    async fn gen_templates(
        &self,
        tables: Vec<String>,
        table_comment_map: HashMap<String, String>,
    ) -> Result<Vec<Table>> {
        let mut templates = vec![];
        for table_name in tables.iter() {
            let sql = format!("SELECT a.attname as name, format_type(a.atttypid,a.atttypmod) as type, a.attnotnull as is_null, col_description(a.attrelid,a.attnum) as comment
FROM pg_class as c,pg_attribute as a where c.relname = '{}' and a.attrelid = c.oid and a.attnum>0", table_name);
            let rows = self.client.query(&sql, &[]).await?;
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
                sql_rows: rows,
                table_comment,
            };
            let table = async_ok!(self.gen_template_data(gen_template_data))?;
            // todo table.index_key = self.index_key(&mut conn, table_name).await?;
            templates.push(table);
        }
        Ok(templates)
    }
}
