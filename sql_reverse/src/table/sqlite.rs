use fn_macro::if_else;
use inflector::Inflector;
use regex::Regex;
use sqlx::{FromRow, Row};
use sqlx::sqlite::SqliteRow;
use crate::reverse_struct::sqlite_impl;
use crate::table::{Field, Table2Comment};
use crate::template::kit::Kit;

#[derive(Debug)]
pub struct Fields {
    pub fields: Vec<Field>,
    pub keys: Vec<String>
}
impl Kit for Fields {}
impl Fields {
    pub fn parse_sql(sql: &str) -> Self {
        let items:Vec<&str> = sql.split("(").last().unwrap().split(")").next().unwrap().split(",").collect();
        let mut fields = vec![];
        let mut keys = vec![];
        let re = Regex::new(r"\s*(\w+)\s+(\w+)\s*").unwrap();
        for item in items {
            if re.is_match(item) {
                let data=  re.captures(item).unwrap();
                let mut data = data.iter();
                data.next();
                let field_name = data.next().unwrap().unwrap().as_str();
                let database_field_type = data.next().unwrap().unwrap().as_str();
                let field_name_camel_case = field_name.to_camel_case();
                let first_char_uppercase_field_name = Self::first_char_to_uppercase(&field_name_camel_case);
                let field_type =
                    Self::get_field_type(&database_field_type, &field_name, &sqlite_impl::FIELD_TYPE.read().unwrap()).unwrap_or_default();
                let is_null = if_else!(item.contains("NOT NULL") || item.contains("not null"), 0, 1);
                if item.contains("primary key") || item.contains("PRIMARY KEY") || item.contains("autoincrement") || item.contains("AUTOINCREMENT") {
                    keys.push(field_name.to_string())
                }
                fields.push(Field {
                    field_name: field_name.to_string(),
                    FieldName: first_char_uppercase_field_name.to_string(),
                    fieldName: field_name_camel_case.to_string(),
                    database_field_type: database_field_type.to_string(),
                    field_type,
                    comment: "".to_string(),
                    is_null,
                    default: None,
                })
            }
        }
        Fields {fields, keys}
    }
}

impl FromRow<'_, SqliteRow> for Fields {
    fn from_row(row: &SqliteRow) -> std::result::Result<Self, sqlx::Error> {
        let sql: String = row.try_get("sql")?;
        Ok(Self::parse_sql(&sql))
    }
}


impl FromRow<'_, SqliteRow> for Table2Comment {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        let table_name = row.try_get("table_name")?;
        Ok(Table2Comment {
            table_name,
            table_comment: None,
            is_key: false,
        })
    }
}