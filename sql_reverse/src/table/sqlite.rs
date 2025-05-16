use fn_macro::if_else;
use inflector::Inflector;
use sqlx::{FromRow, Row};
use sqlx::postgres::PgRow;
use sqlx::sqlite::SqliteRow;
use crate::reverse_struct::postgres_impl;
use crate::reverse_struct::postgres_impl::PostgresStruct;
use crate::table::Field;
use crate::template::kit::Kit;

pub struct Fields {
    pub fields: Vec<Field>
}
impl Kit for Fields {}
impl Fields {
    pub fn parse_sql(sql: &str) -> Self {
        let items:Vec<&str> = sql.split("(").last().unwrap().split(")").next().unwrap().split(",").collect();
        let mut fields = vec![];
        for item in items {
            let data:Vec<&str> = item.trim().split(" ").collect();
            let field_name = data[0];
            let database_field_type = data[1];
            let field_name_camel_case = field_name.clone().to_camel_case();
            let first_char_uppercase_field_name = Self::first_char_to_uppercase(&field_name_camel_case);
            let field_type =
                Self::get_field_type(&database_field_type, &field_name, &postgres_impl::FIELD_TYPE.read().unwrap()).unwrap_or_default();
            let is_null = if_else!(item.contains("NOT NULL"), 0, 1);
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
        Fields {fields}
    }
}

impl FromRow<'_, SqliteRow> for Fields {
    fn from_row(row: &SqliteRow) -> std::result::Result<Self, sqlx::Error> {
        let sql: String = row.try_get("sql")?;
        Ok(Self::parse_sql(&sql))
    }
}