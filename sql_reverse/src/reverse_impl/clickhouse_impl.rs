use crate::error::Result;
use crate::keywords::LANGUAGE;
use crate::reverse_impl::common::CustomConfig;
use crate::reverse_impl::gen_struct::GenStruct;
use crate::table::clickhouse::{ClickhouseField, ClickhouseTable};
use crate::table::{Field, Table, Table2Comment};
use crate::template::kit::Kit;
use fn_macro::btreemap;
use inflector::Inflector;
use std::collections::BTreeMap;
use std::sync::{LazyLock, RwLock};

pub static FIELD_TYPE: LazyLock<RwLock<BTreeMap<String, String>>> = LazyLock::new(|| {
    let map = btreemap!(
        // Integer types
        "^Int8$".to_string() => "i8".to_string(),
        "^Int16$".to_string() => "i16".to_string(),
        "^Int32$".to_string() => "i32".to_string(),
        "^Int64$".to_string() => "i64".to_string(),
        "^Int128$".to_string() => "i128".to_string(),
        "^Int256$".to_string() => "String".to_string(),
        "^UInt8$".to_string() => "u8".to_string(),
        "^UInt16$".to_string() => "u16".to_string(),
        "^UInt32$".to_string() => "u32".to_string(),
        "^UInt64$".to_string() => "u64".to_string(),
        "^UInt128$".to_string() => "u128".to_string(),
        "^UInt256$".to_string() => "String".to_string(),
        // Float types
        "^Float32$".to_string() => "f32".to_string(),
        "^Float64$".to_string() => "f64".to_string(),
        // Decimal types
        "^Decimal".to_string() => "String".to_string(),
        // Boolean
        "^Bool$".to_string() => "bool".to_string(),
        // String types
        "^String$".to_string() => "String".to_string(),
        "^FixedString\\(\\d+\\)$".to_string() => "String".to_string(),
        // Date/Time types
        "^Date$".to_string() => "chrono::NaiveDate".to_string(),
        "^Date32$".to_string() => "chrono::NaiveDate".to_string(),
        "^DateTime$".to_string() => "chrono::NaiveDateTime".to_string(),
        "^DateTime\\(.*\\)$".to_string() => "chrono::NaiveDateTime".to_string(),
        "^DateTime64".to_string() => "chrono::NaiveDateTime".to_string(),
        // UUID
        "^UUID$".to_string() => "String".to_string(),
        // IP types
        "^IPv4$".to_string() => "String".to_string(),
        "^IPv6$".to_string() => "String".to_string(),
        // Enum types
        "^Enum8\\(.*\\)$".to_string() => "String".to_string(),
        "^Enum16\\(.*\\)$".to_string() => "String".to_string(),
        // Complex types - simplified to String
        "^Array\\(.*\\)$".to_string() => "String".to_string(),
        "^Map\\(.*\\)$".to_string() => "String".to_string(),
        "^Tuple\\(.*\\)$".to_string() => "String".to_string(),
        // JSON types
        "^JSON$".to_string() => "serde_json::Value".to_string(),
        "^Object\\('json'\\)$".to_string() => "serde_json::Value".to_string()
    );
    RwLock::new(map)
});

#[derive(Clone)]
pub struct ClickhouseImpl {
    pub config: CustomConfig,
    pub client: clickhouse::Client,
}

impl std::fmt::Debug for ClickhouseImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClickhouseImpl")
            .field("config", &self.config)
            .finish()
    }
}

impl Kit for ClickhouseImpl {}

impl ClickhouseImpl {
    pub async fn init(config: CustomConfig) -> Result<Self> {
        let client = clickhouse::Client::default().with_url(&config.db_url);
        Ok(Self { config, client })
    }

    /// Strip LowCardinality(...) wrapper
    fn strip_low_cardinality(type_str: &str) -> String {
        type_str
            .strip_prefix("LowCardinality(")
            .and_then(|s| s.strip_suffix(')'))
            .unwrap_or(type_str)
            .to_string()
    }

    /// Strip Nullable(...) wrapper and return (inner_type, is_nullable)
    fn strip_nullable(type_str: &str) -> (String, bool) {
        match type_str
            .strip_prefix("Nullable(")
            .and_then(|s| s.strip_suffix(')'))
        {
            Some(inner) => (inner.to_string(), true),
            None => (type_str.to_string(), false),
        }
    }

    /// Normalize ClickHouse type: strip LowCardinality, then Nullable
    fn normalize_type(type_str: &str) -> (String, bool) {
        let stripped = Self::strip_low_cardinality(type_str);
        Self::strip_nullable(&stripped)
    }

    fn convert_field(ch_field: &ClickhouseField) -> Field {
        let (inner_type, is_nullable) = Self::normalize_type(&ch_field.field_type);
        let field_type = Self::get_field_type(
            &inner_type,
            &ch_field.name,
            &FIELD_TYPE.read().unwrap(),
        )
        .unwrap_or_default();
        let field_name_camel_case = ch_field.name.clone().to_camel_case();
        let first_char_uppercase = Self::first_char_to_uppercase(&field_name_camel_case);
        let default = if ch_field.default_expression.is_empty() {
            None
        } else {
            Some(ch_field.default_expression.clone())
        };
        Field {
            field_name: LANGUAGE.check_field_name(&ch_field.name),
            FieldName: first_char_uppercase,
            fieldName: LANGUAGE.check_field_name(&field_name_camel_case),
            database_field_type: ch_field.field_type.clone(),
            field_type,
            comment: ch_field.comment.clone(),
            is_null: if is_nullable { 1 } else { 0 },
            default,
        }
    }

    /// Build primary keys (mapped to unique_key) and sorting keys (mapped to index_key)
    fn build_keys(ch_fields: &[ClickhouseField]) -> (Vec<Vec<String>>, Vec<Vec<String>>) {
        let mut primary_key_fields = vec![];
        let mut sorting_key_fields = vec![];
        for f in ch_fields {
            if f.is_in_primary_key == 1 {
                primary_key_fields.push(f.name.clone());
            }
            if f.is_in_sorting_key == 1 && f.is_in_primary_key == 0 {
                sorting_key_fields.push(f.name.clone());
            }
        }
        let unique_key = if primary_key_fields.is_empty() {
            vec![]
        } else {
            vec![primary_key_fields]
        };
        let index_key = if sorting_key_fields.is_empty() {
            vec![]
        } else {
            vec![sorting_key_fields]
        };
        (index_key, unique_key)
    }
}

const TABLES_SQL: &str = "SELECT name, comment FROM system.tables WHERE database = ? AND engine NOT IN ('System', 'View') ORDER BY name";
const FIELD_SQL: &str = "SELECT name, type as field_type, comment, default_expression, default_kind, is_in_primary_key, is_in_sorting_key FROM system.columns WHERE database = ? AND table = ? ORDER BY position";

impl GenStruct for ClickhouseImpl {
    async fn get_tables(&self) -> Result<Vec<Table2Comment>> {
        let database = self.config.schemaname.clone().unwrap_or_default();
        let ch_tables: Vec<ClickhouseTable> = self
            .client
            .query(TABLES_SQL)
            .bind(&database)
            .fetch_all()
            .await
            .map_err(crate::error::Error::Clickhouse)?;
        let mut tables: Vec<Table2Comment> = ch_tables
            .into_iter()
            .map(|t| Table2Comment {
                table_name: t.name,
                table_comment: if t.comment.is_empty() {
                    None
                } else {
                    Some(t.comment)
                },
            })
            .collect();
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
        let mut templates = vec![];
        for table in tables {
            let ch_fields: Vec<ClickhouseField> = self
                .client
                .query(FIELD_SQL)
                .bind(&database)
                .bind(&table.table_name)
                .fetch_all()
                .await
                .map_err(crate::error::Error::Clickhouse)?;
            let fields: Vec<Field> = ch_fields.iter().map(|f| Self::convert_field(f)).collect();
            let mut struct_name = table.table_name.clone().to_camel_case();
            struct_name = Self::first_char_to_uppercase(&struct_name);
            let (index_key, unique_key) = Self::build_keys(&ch_fields);
            let table = Table {
                table_name: table.table_name.to_owned(),
                struct_name,
                fields,
                comment: table.table_comment.unwrap_or_default(),
                index_key,
                unique_key,
            };
            templates.push(table);
        }
        Ok(templates)
    }

    async fn index_key(&self, _table_name: &str) -> Result<(Vec<Vec<String>>, Vec<Vec<String>>)> {
        // Keys are handled inline in gen_templates via system.columns
        Ok((vec![], vec![]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::clickhouse::ClickhouseField;

    // ========== strip_low_cardinality tests ==========

    #[test]
    fn test_strip_low_cardinality_basic() {
        assert_eq!(
            ClickhouseImpl::strip_low_cardinality("LowCardinality(String)"),
            "String"
        );
    }

    #[test]
    fn test_strip_low_cardinality_nested_nullable() {
        assert_eq!(
            ClickhouseImpl::strip_low_cardinality("LowCardinality(Nullable(String))"),
            "Nullable(String)"
        );
    }

    #[test]
    fn test_strip_low_cardinality_no_wrapper() {
        assert_eq!(
            ClickhouseImpl::strip_low_cardinality("Int32"),
            "Int32"
        );
    }

    #[test]
    fn test_strip_low_cardinality_fixed_string() {
        assert_eq!(
            ClickhouseImpl::strip_low_cardinality("LowCardinality(FixedString(16))"),
            "FixedString(16)"
        );
    }

    // ========== strip_nullable tests ==========

    #[test]
    fn test_strip_nullable_basic() {
        let (inner, is_null) = ClickhouseImpl::strip_nullable("Nullable(Int32)");
        assert_eq!(inner, "Int32");
        assert!(is_null);
    }

    #[test]
    fn test_strip_nullable_string() {
        let (inner, is_null) = ClickhouseImpl::strip_nullable("Nullable(String)");
        assert_eq!(inner, "String");
        assert!(is_null);
    }

    #[test]
    fn test_strip_nullable_no_wrapper() {
        let (inner, is_null) = ClickhouseImpl::strip_nullable("UInt64");
        assert_eq!(inner, "UInt64");
        assert!(!is_null);
    }

    #[test]
    fn test_strip_nullable_datetime() {
        let (inner, is_null) = ClickhouseImpl::strip_nullable("Nullable(DateTime('UTC'))");
        assert_eq!(inner, "DateTime('UTC')");
        assert!(is_null);
    }

    // ========== normalize_type tests ==========

    #[test]
    fn test_normalize_plain_type() {
        let (inner, is_null) = ClickhouseImpl::normalize_type("Int32");
        assert_eq!(inner, "Int32");
        assert!(!is_null);
    }

    #[test]
    fn test_normalize_nullable() {
        let (inner, is_null) = ClickhouseImpl::normalize_type("Nullable(Float64)");
        assert_eq!(inner, "Float64");
        assert!(is_null);
    }

    #[test]
    fn test_normalize_low_cardinality() {
        let (inner, is_null) = ClickhouseImpl::normalize_type("LowCardinality(String)");
        assert_eq!(inner, "String");
        assert!(!is_null);
    }

    #[test]
    fn test_normalize_low_cardinality_nullable() {
        let (inner, is_null) =
            ClickhouseImpl::normalize_type("LowCardinality(Nullable(String))");
        assert_eq!(inner, "String");
        assert!(is_null);
    }

    #[test]
    fn test_normalize_array() {
        let (inner, is_null) = ClickhouseImpl::normalize_type("Array(Int32)");
        assert_eq!(inner, "Array(Int32)");
        assert!(!is_null);
    }

    // ========== FIELD_TYPE mapping tests ==========

    #[test]
    fn test_field_type_integers() {
        let map = FIELD_TYPE.read().unwrap();
        let cases = vec![
            ("Int8", "i8"),
            ("Int16", "i16"),
            ("Int32", "i32"),
            ("Int64", "i64"),
            ("Int128", "i128"),
            ("Int256", "String"),
            ("UInt8", "u8"),
            ("UInt16", "u16"),
            ("UInt32", "u32"),
            ("UInt64", "u64"),
            ("UInt128", "u128"),
            ("UInt256", "String"),
        ];
        for (ch_type, expected_rust) in cases {
            let result =
                ClickhouseImpl::get_field_type(ch_type, "test_field", &map).unwrap();
            assert_eq!(result, expected_rust, "type mapping failed for {}", ch_type);
        }
    }

    #[test]
    fn test_field_type_floats() {
        let map = FIELD_TYPE.read().unwrap();
        assert_eq!(
            ClickhouseImpl::get_field_type("Float32", "f", &map).unwrap(),
            "f32"
        );
        assert_eq!(
            ClickhouseImpl::get_field_type("Float64", "f", &map).unwrap(),
            "f64"
        );
    }

    #[test]
    fn test_field_type_string_types() {
        let map = FIELD_TYPE.read().unwrap();
        assert_eq!(
            ClickhouseImpl::get_field_type("String", "f", &map).unwrap(),
            "String"
        );
        assert_eq!(
            ClickhouseImpl::get_field_type("FixedString(128)", "f", &map).unwrap(),
            "String"
        );
    }

    #[test]
    fn test_field_type_date_time() {
        let map = FIELD_TYPE.read().unwrap();
        assert_eq!(
            ClickhouseImpl::get_field_type("Date", "f", &map).unwrap(),
            "chrono::NaiveDate"
        );
        assert_eq!(
            ClickhouseImpl::get_field_type("Date32", "f", &map).unwrap(),
            "chrono::NaiveDate"
        );
        assert_eq!(
            ClickhouseImpl::get_field_type("DateTime", "f", &map).unwrap(),
            "chrono::NaiveDateTime"
        );
        assert_eq!(
            ClickhouseImpl::get_field_type("DateTime('UTC')", "f", &map).unwrap(),
            "chrono::NaiveDateTime"
        );
        assert_eq!(
            ClickhouseImpl::get_field_type("DateTime64(3)", "f", &map).unwrap(),
            "chrono::NaiveDateTime"
        );
    }

    #[test]
    fn test_field_type_bool() {
        let map = FIELD_TYPE.read().unwrap();
        assert_eq!(
            ClickhouseImpl::get_field_type("Bool", "f", &map).unwrap(),
            "bool"
        );
    }

    #[test]
    fn test_field_type_special_types() {
        let map = FIELD_TYPE.read().unwrap();
        assert_eq!(
            ClickhouseImpl::get_field_type("UUID", "f", &map).unwrap(),
            "String"
        );
        assert_eq!(
            ClickhouseImpl::get_field_type("IPv4", "f", &map).unwrap(),
            "String"
        );
        assert_eq!(
            ClickhouseImpl::get_field_type("IPv6", "f", &map).unwrap(),
            "String"
        );
        assert_eq!(
            ClickhouseImpl::get_field_type("JSON", "f", &map).unwrap(),
            "serde_json::Value"
        );
    }

    #[test]
    fn test_field_type_complex_types() {
        let map = FIELD_TYPE.read().unwrap();
        assert_eq!(
            ClickhouseImpl::get_field_type("Array(Int32)", "f", &map).unwrap(),
            "String"
        );
        assert_eq!(
            ClickhouseImpl::get_field_type("Map(String, UInt64)", "f", &map).unwrap(),
            "String"
        );
        assert_eq!(
            ClickhouseImpl::get_field_type("Tuple(String, Int32)", "f", &map).unwrap(),
            "String"
        );
        assert_eq!(
            ClickhouseImpl::get_field_type("Enum8('a' = 1, 'b' = 2)", "f", &map).unwrap(),
            "String"
        );
    }

    #[test]
    fn test_field_type_decimal_variants() {
        let map = FIELD_TYPE.read().unwrap();
        assert_eq!(
            ClickhouseImpl::get_field_type("Decimal(10, 2)", "f", &map).unwrap(),
            "String"
        );
        assert_eq!(
            ClickhouseImpl::get_field_type("Decimal32(4)", "f", &map).unwrap(),
            "String"
        );
        assert_eq!(
            ClickhouseImpl::get_field_type("Decimal128(8)", "f", &map).unwrap(),
            "String"
        );
    }

    // ========== convert_field tests ==========

    /// Initialize the LANGUAGE global for tests
    async fn init_language() {
        let _ = crate::keywords::get_or_init("rs").await;
    }

    fn make_ch_field(name: &str, field_type: &str) -> ClickhouseField {
        ClickhouseField {
            name: name.to_string(),
            field_type: field_type.to_string(),
            comment: String::new(),
            default_expression: String::new(),
            default_kind: String::new(),
            is_in_primary_key: 0,
            is_in_sorting_key: 0,
        }
    }

    #[tokio::test]
    async fn test_convert_field_basic_int() {
        init_language().await;
        let ch = make_ch_field("user_id", "UInt64");
        let field = ClickhouseImpl::convert_field(&ch);
        assert_eq!(field.field_name, "user_id");
        assert_eq!(field.field_type, "u64");
        assert_eq!(field.FieldName, "UserId");
        assert_eq!(field.fieldName, "userId");
        assert_eq!(field.database_field_type, "UInt64");
        assert_eq!(field.is_null, 0);
    }

    #[tokio::test]
    async fn test_convert_field_nullable() {
        init_language().await;
        let ch = make_ch_field("email", "Nullable(String)");
        let field = ClickhouseImpl::convert_field(&ch);
        assert_eq!(field.field_name, "email");
        assert_eq!(field.field_type, "String");
        assert_eq!(field.is_null, 1);
        assert_eq!(field.database_field_type, "Nullable(String)");
    }

    #[tokio::test]
    async fn test_convert_field_low_cardinality_nullable() {
        init_language().await;
        let ch = make_ch_field("status", "LowCardinality(Nullable(String))");
        let field = ClickhouseImpl::convert_field(&ch);
        assert_eq!(field.field_type, "String");
        assert_eq!(field.is_null, 1);
    }

    #[tokio::test]
    async fn test_convert_field_with_comment_and_default() {
        init_language().await;
        let ch = ClickhouseField {
            name: "created_at".to_string(),
            field_type: "DateTime".to_string(),
            comment: "creation time".to_string(),
            default_expression: "now()".to_string(),
            default_kind: "DEFAULT".to_string(),
            is_in_primary_key: 0,
            is_in_sorting_key: 0,
        };
        let field = ClickhouseImpl::convert_field(&ch);
        assert_eq!(field.comment, "creation time");
        assert_eq!(field.default, Some("now()".to_string()));
        assert_eq!(field.field_type, "chrono::NaiveDateTime");
    }

    #[tokio::test]
    async fn test_convert_field_empty_default() {
        init_language().await;
        let ch = make_ch_field("name", "String");
        let field = ClickhouseImpl::convert_field(&ch);
        assert_eq!(field.default, None);
    }

    // ========== build_keys tests ==========

    fn make_key_field(name: &str, primary: u8, sorting: u8) -> ClickhouseField {
        ClickhouseField {
            name: name.to_string(),
            field_type: "UInt64".to_string(),
            comment: String::new(),
            default_expression: String::new(),
            default_kind: String::new(),
            is_in_primary_key: primary,
            is_in_sorting_key: sorting,
        }
    }

    #[test]
    fn test_build_keys_primary_only() {
        let fields = vec![
            make_key_field("id", 1, 1),
            make_key_field("name", 0, 0),
        ];
        let (index_key, unique_key) = ClickhouseImpl::build_keys(&fields);
        assert_eq!(unique_key, vec![vec!["id".to_string()]]);
        assert!(index_key.is_empty());
    }

    #[test]
    fn test_build_keys_with_extra_sorting() {
        let fields = vec![
            make_key_field("id", 1, 1),
            make_key_field("date", 0, 1),
            make_key_field("name", 0, 0),
        ];
        let (index_key, unique_key) = ClickhouseImpl::build_keys(&fields);
        assert_eq!(unique_key, vec![vec!["id".to_string()]]);
        assert_eq!(index_key, vec![vec!["date".to_string()]]);
    }

    #[test]
    fn test_build_keys_composite_primary() {
        let fields = vec![
            make_key_field("tenant_id", 1, 1),
            make_key_field("user_id", 1, 1),
            make_key_field("name", 0, 0),
        ];
        let (index_key, unique_key) = ClickhouseImpl::build_keys(&fields);
        assert_eq!(
            unique_key,
            vec![vec!["tenant_id".to_string(), "user_id".to_string()]]
        );
        assert!(index_key.is_empty());
    }

    #[test]
    fn test_build_keys_no_keys() {
        let fields = vec![
            make_key_field("col1", 0, 0),
            make_key_field("col2", 0, 0),
        ];
        let (index_key, unique_key) = ClickhouseImpl::build_keys(&fields);
        assert!(unique_key.is_empty());
        assert!(index_key.is_empty());
    }

    #[test]
    fn test_build_keys_empty_fields() {
        let fields: Vec<ClickhouseField> = vec![];
        let (index_key, unique_key) = ClickhouseImpl::build_keys(&fields);
        assert!(unique_key.is_empty());
        assert!(index_key.is_empty());
    }
}
