use crate::error::Result;
use crate::keywords::LANGUAGE;
use crate::reverse_impl::common::CustomConfig;
use crate::reverse_impl::gen_struct::GenStruct;
use crate::table::tdengine::{TdengineDescribeRow, TdengineNormalTable, TdengineStable};
use crate::table::{Field, Table, Table2Comment};
use crate::template::kit::Kit;
use fn_macro::btreemap;
use futures::TryStreamExt;
use inflector::Inflector;
use std::collections::BTreeMap;
use std::sync::{LazyLock, RwLock};
use taos::{AsyncFetchable, AsyncQueryable, AsyncTBuilder};

pub static FIELD_TYPE: LazyLock<RwLock<BTreeMap<String, String>>> = LazyLock::new(|| {
    let map = btreemap!(
        // Timestamp
        "^TIMESTAMP$".to_string() => "chrono::NaiveDateTime".to_string(),
        // Boolean
        "^BOOL$".to_string() => "bool".to_string(),
        // Signed integers
        "^TINYINT$".to_string() => "i8".to_string(),
        "^SMALLINT$".to_string() => "i16".to_string(),
        "^INT$".to_string() => "i32".to_string(),
        "^BIGINT$".to_string() => "i64".to_string(),
        // Unsigned integers
        "^TINYINT UNSIGNED$".to_string() => "u8".to_string(),
        "^SMALLINT UNSIGNED$".to_string() => "u16".to_string(),
        "^INT UNSIGNED$".to_string() => "u32".to_string(),
        "^BIGINT UNSIGNED$".to_string() => "u64".to_string(),
        // Float types
        "^FLOAT$".to_string() => "f32".to_string(),
        "^DOUBLE$".to_string() => "f64".to_string(),
        // String types
        "^BINARY.*$".to_string() => "String".to_string(),
        "^VARCHAR.*$".to_string() => "String".to_string(),
        "^NCHAR.*$".to_string() => "String".to_string(),
        // Binary data types
        "^VARBINARY.*$".to_string() => "String".to_string(),
        "^BLOB$".to_string() => "String".to_string(),
        "^GEOMETRY.*$".to_string() => "String".to_string(),
        // JSON type
        "^JSON$".to_string() => "serde_json::Value".to_string(),
        // Decimal type
        "^DECIMAL.*$".to_string() => "String".to_string()
    );
    RwLock::new(map)
});

#[derive(Clone)]
pub struct TdengineImpl {
    pub config: CustomConfig,
    pub builder: taos::TaosBuilder,
}

impl std::fmt::Debug for TdengineImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TdengineImpl")
            .field("config", &self.config)
            .finish()
    }
}

impl Kit for TdengineImpl {}

impl TdengineImpl {
    pub async fn init(config: CustomConfig) -> Result<Self> {
        let builder = taos::TaosBuilder::from_dsn(&config.db_url)?;
        Ok(Self { config, builder })
    }

    async fn connect(&self) -> Result<taos::Taos> {
        Ok(self.builder.build().await?)
    }

    fn convert_field(row: &TdengineDescribeRow) -> Field {
        let field_type = Self::get_field_type(
            &row.field_type,
            &row.field,
            &FIELD_TYPE.read().unwrap(),
        )
        .unwrap_or_default();
        let field_name_camel_case = row.field.clone().to_camel_case();
        let first_char_uppercase = Self::first_char_to_uppercase(&field_name_camel_case);
        // In TDengine, TIMESTAMP (first column) is NOT NULL; all other columns are nullable
        let is_null = if row.field_type == "TIMESTAMP" { 0 } else { 1 };
        let comment = if row.note == "TAG" {
            "[TAG]".to_string()
        } else {
            String::new()
        };
        Field {
            field_name: LANGUAGE.check_field_name(&row.field),
            FieldName: first_char_uppercase,
            fieldName: LANGUAGE.check_field_name(&field_name_camel_case),
            database_field_type: row.field_type.clone(),
            field_type,
            comment,
            is_null: is_null,
            default: None,
        }
    }

    /// Build tag columns as unique_key (similar to ClickHouse primary key mapping)
    fn build_tag_keys(rows: &[TdengineDescribeRow]) -> Vec<Vec<String>> {
        let tag_fields: Vec<String> = rows
            .iter()
            .filter(|r| r.note == "TAG")
            .map(|r| r.field.clone())
            .collect();
        if tag_fields.is_empty() {
            vec![]
        } else {
            vec![tag_fields]
        }
    }
}

impl GenStruct for TdengineImpl {
    async fn get_tables(&self) -> Result<Vec<Table2Comment>> {
        let database = self.config.schemaname.clone().unwrap_or_default();
        let taos = self.connect().await?;

        // Query supertables
        let stables_sql = format!(
            "SELECT stable_name, table_comment FROM information_schema.ins_stables WHERE db_name = '{}'",
            database
        );
        let stables: Vec<TdengineStable> = taos
            .query(&stables_sql)
            .await
            .map_err(crate::error::Error::Taos)?
            .deserialize()
            .try_collect()
            .await
            .map_err(crate::error::Error::Taos)?;

        // Query normal tables (not child tables, not supertables)
        let tables_sql = format!(
            "SELECT table_name, table_comment FROM information_schema.ins_tables WHERE db_name = '{}' AND type = 'NORMAL_TABLE'",
            database
        );
        let normal_tables: Vec<TdengineNormalTable> = taos
            .query(&tables_sql)
            .await
            .map_err(crate::error::Error::Taos)?
            .deserialize()
            .try_collect()
            .await
            .map_err(crate::error::Error::Taos)?;

        let mut tables: Vec<Table2Comment> = Vec::new();

        // Add supertables
        for st in stables {
            tables.push(Table2Comment {
                table_name: st.stable_name,
                table_comment: st.table_comment,
            });
        }

        // Add normal tables
        for nt in normal_tables {
            tables.push(Table2Comment {
                table_name: nt.table_name,
                table_comment: nt.table_comment,
            });
        }

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
        let database = self.config.schemaname.clone().unwrap_or_default();
        let taos = self.connect().await?;
        let mut templates = vec![];
        for table in tables {
            let describe_sql = format!("DESCRIBE `{}`.`{}`", database, table.table_name);
            let rows: Vec<TdengineDescribeRow> = taos
                .query(&describe_sql)
                .await
                .map_err(crate::error::Error::Taos)?
                .deserialize()
                .try_collect()
                .await
                .map_err(crate::error::Error::Taos)?;

            let fields: Vec<Field> = rows.iter().map(|r| Self::convert_field(r)).collect();
            let mut struct_name = table.table_name.clone().to_camel_case();
            struct_name = Self::first_char_to_uppercase(&struct_name);
            let unique_key = Self::build_tag_keys(&rows);

            let table = Table {
                table_name: table.table_name.to_owned(),
                struct_name,
                fields,
                comment: table.table_comment.unwrap_or_default(),
                index_key: vec![],
                unique_key,
            };
            templates.push(table);
        }
        Ok(templates)
    }

    async fn index_key(&self, _table_name: &str) -> Result<(Vec<Vec<String>>, Vec<Vec<String>>)> {
        // Keys are handled inline in gen_templates via DESCRIBE
        Ok((vec![], vec![]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::tdengine::TdengineDescribeRow;

    // ========== FIELD_TYPE mapping tests ==========

    #[test]
    fn test_field_type_timestamp() {
        let map = FIELD_TYPE.read().unwrap();
        assert_eq!(
            TdengineImpl::get_field_type("TIMESTAMP", "ts", &map).unwrap(),
            "chrono::NaiveDateTime"
        );
    }

    #[test]
    fn test_field_type_bool() {
        let map = FIELD_TYPE.read().unwrap();
        assert_eq!(
            TdengineImpl::get_field_type("BOOL", "f", &map).unwrap(),
            "bool"
        );
    }

    #[test]
    fn test_field_type_signed_integers() {
        let map = FIELD_TYPE.read().unwrap();
        let cases = vec![
            ("TINYINT", "i8"),
            ("SMALLINT", "i16"),
            ("INT", "i32"),
            ("BIGINT", "i64"),
        ];
        for (td_type, expected_rust) in cases {
            let result =
                TdengineImpl::get_field_type(td_type, "test_field", &map).unwrap();
            assert_eq!(result, expected_rust, "type mapping failed for {}", td_type);
        }
    }

    #[test]
    fn test_field_type_unsigned_integers() {
        let map = FIELD_TYPE.read().unwrap();
        let cases = vec![
            ("TINYINT UNSIGNED", "u8"),
            ("SMALLINT UNSIGNED", "u16"),
            ("INT UNSIGNED", "u32"),
            ("BIGINT UNSIGNED", "u64"),
        ];
        for (td_type, expected_rust) in cases {
            let result =
                TdengineImpl::get_field_type(td_type, "test_field", &map).unwrap();
            assert_eq!(result, expected_rust, "type mapping failed for {}", td_type);
        }
    }

    #[test]
    fn test_field_type_floats() {
        let map = FIELD_TYPE.read().unwrap();
        assert_eq!(
            TdengineImpl::get_field_type("FLOAT", "f", &map).unwrap(),
            "f32"
        );
        assert_eq!(
            TdengineImpl::get_field_type("DOUBLE", "f", &map).unwrap(),
            "f64"
        );
    }

    #[test]
    fn test_field_type_string_types() {
        let map = FIELD_TYPE.read().unwrap();
        assert_eq!(
            TdengineImpl::get_field_type("BINARY", "f", &map).unwrap(),
            "String"
        );
        assert_eq!(
            TdengineImpl::get_field_type("VARCHAR", "f", &map).unwrap(),
            "String"
        );
        assert_eq!(
            TdengineImpl::get_field_type("NCHAR", "f", &map).unwrap(),
            "String"
        );
    }

    #[test]
    fn test_field_type_json() {
        let map = FIELD_TYPE.read().unwrap();
        assert_eq!(
            TdengineImpl::get_field_type("JSON", "f", &map).unwrap(),
            "serde_json::Value"
        );
    }

    #[test]
    fn test_field_type_binary_data() {
        let map = FIELD_TYPE.read().unwrap();
        assert_eq!(
            TdengineImpl::get_field_type("VARBINARY", "f", &map).unwrap(),
            "String"
        );
        assert_eq!(
            TdengineImpl::get_field_type("BLOB", "f", &map).unwrap(),
            "String"
        );
        assert_eq!(
            TdengineImpl::get_field_type("GEOMETRY", "f", &map).unwrap(),
            "String"
        );
    }

    // ========== convert_field tests ==========

    /// Initialize the LANGUAGE global for tests
    async fn init_language() {
        let _ = crate::keywords::get_or_init("rs").await;
    }

    fn make_describe_row(field: &str, field_type: &str, note: &str) -> TdengineDescribeRow {
        TdengineDescribeRow {
            field: field.to_string(),
            field_type: field_type.to_string(),
            length: 0,
            note: note.to_string(),
        }
    }

    #[tokio::test]
    async fn test_convert_field_timestamp() {
        init_language().await;
        let row = make_describe_row("ts", "TIMESTAMP", "");
        let field = TdengineImpl::convert_field(&row);
        assert_eq!(field.field_name, "ts");
        assert_eq!(field.field_type, "chrono::NaiveDateTime");
        assert_eq!(field.is_null, 0); // TIMESTAMP is NOT NULL
        assert_eq!(field.comment, "");
    }

    #[tokio::test]
    async fn test_convert_field_regular_column() {
        init_language().await;
        let row = make_describe_row("current", "FLOAT", "");
        let field = TdengineImpl::convert_field(&row);
        assert_eq!(field.field_name, "current");
        assert_eq!(field.field_type, "f32");
        assert_eq!(field.is_null, 1); // Regular columns are nullable
        assert_eq!(field.comment, "");
    }

    #[tokio::test]
    async fn test_convert_field_tag_column() {
        init_language().await;
        let row = make_describe_row("location", "NCHAR", "TAG");
        let field = TdengineImpl::convert_field(&row);
        assert_eq!(field.field_name, "location");
        assert_eq!(field.field_type, "String");
        assert_eq!(field.is_null, 1);
        assert_eq!(field.comment, "[TAG]");
    }

    #[tokio::test]
    async fn test_convert_field_camel_case() {
        init_language().await;
        let row = make_describe_row("group_id", "INT", "TAG");
        let field = TdengineImpl::convert_field(&row);
        assert_eq!(field.field_name, "group_id");
        assert_eq!(field.FieldName, "GroupId");
        assert_eq!(field.fieldName, "groupId");
    }

    // ========== build_tag_keys tests ==========

    #[test]
    fn test_build_tag_keys_with_tags() {
        let rows = vec![
            make_describe_row("ts", "TIMESTAMP", ""),
            make_describe_row("current", "FLOAT", ""),
            make_describe_row("location", "NCHAR", "TAG"),
            make_describe_row("group_id", "INT", "TAG"),
        ];
        let keys = TdengineImpl::build_tag_keys(&rows);
        assert_eq!(
            keys,
            vec![vec!["location".to_string(), "group_id".to_string()]]
        );
    }

    #[test]
    fn test_build_tag_keys_no_tags() {
        let rows = vec![
            make_describe_row("ts", "TIMESTAMP", ""),
            make_describe_row("value", "DOUBLE", ""),
        ];
        let keys = TdengineImpl::build_tag_keys(&rows);
        assert!(keys.is_empty());
    }

    #[test]
    fn test_build_tag_keys_empty() {
        let rows: Vec<TdengineDescribeRow> = vec![];
        let keys = TdengineImpl::build_tag_keys(&rows);
        assert!(keys.is_empty());
    }
}
