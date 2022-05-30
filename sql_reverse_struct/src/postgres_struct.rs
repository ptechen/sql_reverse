use crate::common::CustomConfig;
use crate::gen_struct::GenStruct;
use async_trait::async_trait;
use inflector::Inflector;
use once_cell::sync::Lazy;
use sql_reverse_error::result::Result;
use sql_reverse_template::table::{Field, Table};
use std::collections::{HashMap, BTreeMap};
use tokio_postgres::{Client, NoTls, Row};
use regex::Regex;

pub static FIELD_TYPE: Lazy<BTreeMap<String, String>> = Lazy::new(|| {
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
    map.insert(r"^timestamp with time zone$".to_string(), "NaiveDateTime".to_string());
    map.insert(r"^time with time zone$".to_string(), "NaiveDateTime".to_string());
    map.insert(r"^time$".to_string(), "NaiveDateTime".to_string());
    map.insert(r"^date$".to_string(), "Date".to_string());
    map.insert(r"^interval$".to_string(), "String".to_string());
    map.insert(r"^uuid$".to_string(), "String".to_string());
    map.insert(r"^xml$".to_string(), "String".to_string());
    map.insert(r"^json$".to_string(), "String".to_string());
    map.insert(r"^jsonb$".to_string(), "String".to_string());
    map.insert(r"^jsonpath$".to_string(), "String".to_string());
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
            .port(config.port)
            .connect(NoTls)
            .await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                panic!("connection error: {}", e);
            }
        });
        Ok(Self { config, client })
    }

    async fn gen_template_data(&self, gen_template_data: GenTemplateData, fields_type: &Option<BTreeMap<String, String>>) -> Result<Table> {
        let default_type = FIELD_TYPE.clone();
        let fields_type = fields_type.as_ref().unwrap_or(&default_type);
        let mut fields = vec![];
        for row in gen_template_data.sql_rows.iter() {
            let field_name: String = row.get(0);
            let camel_name: String = field_name.to_camel_case();
            let capitalized_camel_case = async_ok!(self.first_char_to_uppercase(&camel_name))?;
            if field_name.contains("drop") {
                continue;
            }
            let database_field_type: String = row.get(1);
            let field_type = async_ok!(self.get_field_type(&database_field_type, fields_type))?;
            let is_null: bool = row.get(2);
            let mut cur_is_null = 0;
            if is_null == false {
                cur_is_null = 1;
            }
            let comment: Option<String> = row.get(3);
            let default: Option<String> = row.get(4);
            let field = Field {
                field_name,
                FieldName: capitalized_camel_case,
                fieldName: camel_name,
                database_field_type,
                field_type,
                comment: comment.unwrap_or_default(),
                is_null: cur_is_null,
                default,
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

    async fn index_key(&self, table_name: &str) -> Result<Vec<Vec<String>>> {
        let sql = format!("SELECT indexdef FROM pg_indexes WHERE schemaname = '{}' and tablename = '{}'", self.config.schemaname.to_owned().unwrap_or_default(), table_name);
        let rows = &self
            .client
            .query(
                &sql,
                &[],
            )
            .await?;
        let mut index_list = vec![];
        let re = Regex::new("\\(.*\\)")?;
        for row in rows {
            let value: &str = row.get(0);
            if re.is_match(value) {
                let data = re.find(value).unwrap();
                let v:&[u8] = value.as_bytes().get(data.start() + 1..data.end() - 1).unwrap();
                let v = String::from_utf8_lossy(v).to_string();
                let v:Vec<&str> = v.split(", ").collect();
                let mut index = vec![];
                for v in v {
                    index.push(v.to_owned());
                }
                index_list.push(index);
            }
        }
        Ok(index_list)
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
        fields_type: Option<BTreeMap<String, String>>,
    ) -> Result<Vec<Table>> {
        let mut templates = vec![];
        for table_name in tables.iter() {
            let sql = format!("select a.attname                 as filed,
        format_type(a.atttypid, a.atttypmod)                            as filed_type,
        a.attnotnull                                                    as is_null,
        col_description(a.attrelid, a.attnum)                           as comment,
        pg_get_expr(d.adbin, d.adrelid)                                 as default_value
        from pg_class c,
        pg_attribute a
        left join (select a.attname, ad.adbin, ad.adrelid
                  from pg_class c,
                       pg_attribute a,
                       pg_attrdef ad
                  where relname = '{}'
                    and ad.adrelid = c.oid
                    and adnum = a.attnum
                    and attrelid = c.oid) as d on a.attname = d.attname
        where c.relname = '{}'
        and a.attrelid = c.oid
        and a.attnum > 0", table_name, table_name);
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
            let mut table = async_ok!(self.gen_template_data(gen_template_data, &fields_type))?;
            table.index_key = self.index_key(table_name).await?;
            templates.push(table);
        }
        Ok(templates)
    }
}
