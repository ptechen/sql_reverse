use crate::reverse_struct::mysql_impl;
use crate::table::{Field, Table2Comment};
use crate::template::kit::Kit;
use inflector::Inflector;
use sqlx::mysql::MySqlRow;
use sqlx::{FromRow, Row};

impl FromRow<'_, MySqlRow> for Field {
    fn from_row(row: &MySqlRow) -> Result<Self, sqlx::Error> {
        let field_name: String = row.try_get("field_name")?;
        let database_field_type: String = row.try_get("field_type")?;
        let comment: String = row.try_get("comment").unwrap_or_default();
        let is_null: i64 = row.try_get("is_null").unwrap_or_default();
        let field_name_camel_case = field_name.clone().to_camel_case();
        let first_char_uppercase_field_name = Self::first_char_to_uppercase(&field_name_camel_case);

        let field_type = Self::get_field_type(
            &database_field_type,
            &field_name,
            &mysql_impl::FIELD_TYPE.read().unwrap(),
        )
        .unwrap_or_default();
        let default = row.try_get("default_value").ok();
        Ok(Field {
            field_name,
            FieldName: first_char_uppercase_field_name,
            fieldName: field_name_camel_case,
            database_field_type,
            field_type,
            comment,
            is_null: is_null as u8,
            default,
        })
    }
}

impl FromRow<'_, MySqlRow> for Table2Comment {
    fn from_row(row: &MySqlRow) -> Result<Self, sqlx::Error> {
        let table_name = row.try_get("table_name")?;
        let table_comment = row.try_get("table_comment")?;
        Ok(Table2Comment {
            table_name,
            table_comment,
        })
    }
}
