# sql reverse

# Generate the multiple programming languages structure based on the MySQL/PostgresSQL table structure
[![Version info](https://img.shields.io/crates/v/sql_reverse.svg)](https://crates.io/crates/sql_reverse)
[![Downloads](https://img.shields.io/crates/d/sql_reverse.svg?style=flat-square)](https://crates.io/crates/sql_reverse)
[![docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/sql_reverse)
[![dependency status](https://deps.rs/crate/sql_reverse/0.1.0/status.svg)](https://deps.rs/crate/sql_reverse)
[![B站](https://img.shields.io/badge/B站-latest-blue.svg?style=flat-square)](https://www.bilibili.com/video/BV1dF411L7P7?spm_id_from=333.337.search-card.all.click)

## Install
    cargo install sql_reverse

## sql_reverse <SUBCOMMAND>
    USAGE:
    sql_reverse <SUBCOMMAND>
    
    FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    
    SUBCOMMANDS:
    export      Export default database field types
    help        Prints this message or the help of the given subcommand(s)
    mysql       Mysql OPTIONS
    postgres    PostgresSQL OPTIONS



## sql_reverse mysql/postgres [OPTIONS]
    USAGE:
    sql_reverse mysql/postgres [OPTIONS]
    
    FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    
    OPTIONS:
    -c <custom-field-type>        Custom field type [default: ./default.json]
    -f <file>                     Input config file to read [default: ./reverse.yml]
    -s <suffix>                   Suffix of the generated file [default: rs]
    -n <template-name>            Input template name [default: base.tera]
    -p <template-path>            Input template path [default: templates/*]


## Exec，you need to make sure you're in the same directory as templates.
    sql_reverse export
    sql_reverse mysql -f reverse.yml
    sql_reverse postgres -f reverse.yml
## Custom Exec
    sql_reverse export
    sql_reverse mysql -f reverse.yml -p 'templates/*' -s rs -n base.tera -c ./mysql_default.json
    sql_reverse postgres -f reverse.yml -p 'templates/*' -s rs -n base.tera -c ./postgres_default.json
## reverse.yml
    host: 127.0.0.1
    port: 3306
    username: root
    password: ''
    database: db_name
    schemaname: test # only postgres enable
    include_tables: # Include tables, can be ignored.
    #  - table_name
    exclude_tables: # Exclude, tables, can be ignored.
    #  - table_name
    output_dir: ./dir # code output directory

## Template Struct:
    /// sql 表
    #[derive(Serialize, Clone, Default, Debug)]
    pub struct Table {
        /// 表名
        pub table_name: String,
        /// 结构体名
        pub struct_name: String,
        /// 字段列表
        pub fields: Vec<Field>,
        /// 表注释
        pub comment: String,
        /// 索引
        pub index_key: Vec<Vec<String>>,
    }
    
    /// sql 字段
    #[allow(non_snake_case)]
    #[derive(Serialize, Clone, Default, Debug)]
    pub struct Field {
        /// 数据库字段名
        pub field_name: String,
        /// 首字母大写驼峰字段名
        pub FieldName: String,
        /// 首字母小写驼峰字段名
        pub fieldName: String,
        /// 数据库字段类型
        pub database_field_type: String,
        /// 字段类型
        pub field_type: String,
        /// 注释
        pub comment: String,
        /// 默认值是否为null, 1: 是 0: 不是
        pub is_null: u8,
        /// 默认值
        pub default: Option<String>
    }

## Template:
    use serde_derive;
    use chrono::prelude::*;
    use serde::{Deserialize, Serialize};
    
    {% if table.comment -%}
    	/// {{ table.comment }}
    {% endif -%}
    {% for index in table.index_key -%}
        /// 索引：{{index}}
    {% endfor -%}
    #[crud_table]
    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct {{ table.struct_name }} {
    {%- for v in table.fields %}
    	{% if v.comment -%}
    	    /// {{ v.comment }} {% if v.database_field_type %} field_type: {{ v.database_field_type }}{% endif %}{% if v.default %} default: {{ v.default }}{% endif %} {% if v.default == '' %} default: ''{% endif %}
    	{% endif -%}
    	{% if v.is_null == 1 -%}
        	pub {{ v.field_name }}: Option<{{ v.field_type }}>,
        {%- else -%}
            {% if v.field_type == 'NaiveDateTime' -%}
                pub {{ v.field_name }}: Option<{{ v.field_type }}>,
            {%- else -%}
                pub {{ v.field_name }}: {{ v.field_type }},
            {%- endif -%}
        {%- endif -%}
    {%- endfor %}
    }
## Gen Struct Example:
	use serde_derive;
	use chrono::prelude::*;
	use serde::{Deserialize, Serialize};
	
	/// 用户信息
	/// 索引：[article_id]
	#[crud_table]
	#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
	pub struct Article {
		pub article_id: u64,
		/// 用户id  field_type: varchar(16)  default: ''
		pub user_id: Option<String>,
		/// 用户类型  field_type: tinyint default: 0 
		pub user_type: i8,
		/// 文章名  field_type: varchar(32)  default: ''
		pub article_title: String,
		/// 内容简述  field_type: text 
		pub article_content: String,
		/// 头像  field_type: varchar(128)  default: ''
		pub article_url: String,
		/// 点赞数  field_type: int unsigned default: 0 
		pub likes: u32,
		pub is_deleted: u8,
		pub updated_at: Option<NaiveDateTime>,
		pub created_at: Option<NaiveDateTime>,
	}
