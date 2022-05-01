# sql_reveser

# Generate the RUST structure based on the MySQL table structure
[![Version info](https://img.shields.io/crates/v/sql_reveser.svg)](https://crates.io/crates/sql_reveser)
[![Downloads](https://img.shields.io/crates/d/sql_reveser.svg?style=flat-square)](https://crates.io/crates/sql_reveser)
[![docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/sql_reveser)
[![dependency status](https://deps.rs/crate/sql_reveser/0.1.0/status.svg)](https://deps.rs/crate/sql_reveser)

## Install
    cargo install sql_reveser

## Exec，you need to make sure you're in the same directory as templates.
    sql_reveser mysql -f reverse.yml
## Custom Exec
    sql_reveser mysql -f reverse.yml -p 'templates/*' -n base.tera

## reverse.yml
    host: 127.0.0.1
    post: 3306
    username: root
    password: ''
    database: db_name
    include_tables: # Include tables, can be ignored.
    #  - table_name
    exclude_tables: # Exclude, tables, can be ignored.
    #  - table_name
    output_dir: ./dir # code output directory

## Template Struct:
    #[derive(Serialize)]
    pub struct Template {
        pub table_name: String,
        pub struct_name: String,
        pub fields: Vec<Field>, 
        pub comment: String,
    }

    #[derive(Serialize, Clone)]
    pub struct Field {
        pub field_name: String,
        pub field_type: String,
        pub comment: String,
        /// 1: 是, 0: 否
        pub is_null: u8,
    }

## Template:
    use serde_derive;
    use chrono::prelude::*;

    {% if template.comment -%}
        /// {{ template.comment }}
    {% endif -%}
    #[crud_table]
    #[derive(Default, Debug, Clone, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
    pub struct {{ template.struct_name }} {
    {%- for v in template.fields %}
        {% if v.comment -%}
            /// {{ v.comment }}
        {% endif -%}
        {% if v.is_null == 1 -%}
            pub {{ v.field_name }}: Option<{{ v.field_type }}>,
        {%- else -%}
            pub {{ v.field_name }}: {{ v.field_type }},
        {%- endif -%}
    {%- endfor %}
    }

## Gen Struct Example:
    use serde_derive;
    use chrono::prelude::*;
    
    /// Test
    #[crud_table]
    #[derive(Default, Debug, Clone, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)
    pub struct Test {
        pub id: Option<u32>,
        /// uuid
        pub uuid: Option<String>,
        /// 数据
        pub content: Option<String>,
        /// 版本
        pub version: Option<i8>,
        /// 1:删除, 0:未删除
        pub is_deleted: Option<u8>,
        /// 更新时间
        pub updated_at: Option<NaiveDateTime>,
        /// 创建时间
        pub created_at: Option<NaiveDateTime>,
    }
