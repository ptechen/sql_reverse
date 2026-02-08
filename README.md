# sql reverse

# 基于数据库表结构自定义模版生成多种编程语言代码的命令行工具，支持 MySQL、Postgres、Sqlite、ClickHouse。
# A command-line tool that generates codes in multiple programming languages based on custom templates of database table structures, supporting MySQL\Postgres\Sqlite\ClickHouse.
[![Version info](https://img.shields.io/crates/v/sql_reverse.svg)](https://crates.io/crates/sql_reverse)
[![Downloads](https://img.shields.io/crates/d/sql_reverse.svg?style=flat-square)](https://crates.io/crates/sql_reverse)
[![docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/sql_reverse)
[![dependency status](https://deps.rs/crate/sql_reverse/0.1.0/status.svg)](https://deps.rs/crate/sql_reverse)

## Install
    cargo install sql_reverse

## sql_reverse <SUBCOMMAND>
    classify 0.1.13
    
    USAGE:
        sql_reverse <SUBCOMMAND>
    
    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information
    
    SUBCOMMANDS:
        clickhouse    Clickhouse OPTIONS
        export        Export default database field types
        help          Prints this message or the help of the given subcommand(s)
        mysql         Mysql OPTIONS
        postgres      Postgres OPTIONS
        sqlite        Sqlite OPTIONS




## sql_reverse mysql/postgres/sqlite/clickhouse [OPTIONS]
    USAGE:
    sql_reverse mysql/postgres/sqlite/clickhouse [OPTIONS]
    
    FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    
    OPTIONS:
    -c <custom-field-type>        Custom field type, example: -c ./default.json [default: ]
    -f <file>                     Input database config file to read, example: -f ./reverse.yml [default: ./reverse.yml]
    -s <suffix>                   Suffix of the generated file, example: -s rs [default: rs]
    -n <template-name>            Input template name, example: -n base.tera [default: base.tera]
    -p <template-path>            Input template path example: -p 'templates/*' [default: templates/*]



## Exec，you need to make sure you're in the same directory as templates.
    sql_reverse export
    sql_reverse mysql -f reverse.yml
    sql_reverse postgres -f reverse.yml
    sql_reverse sqlite -f reverse.yml
    sql_reverse clickhouse -f reverse.yml
## Custom Exec
    sql_reverse export
    sql_reverse mysql -f reverse.yml -p 'templates/*' -s rs -n mysql.tera -c ./mysql_default.json
    sql_reverse postgres -f reverse.yml -p 'templates/*' -s rs -n postgres.tera -c ./postgres_default.json
    sql_reverse sqlite -f reverse.yml -p 'templates/*' -s rs -n sqlite.tera -c ./sqlite_default.json
    sql_reverse clickhouse -f reverse.yml -p 'templates/*' -s rs -n clickhouse.tera -c ./clickhouse_default.json
## reverse.yml
    # MySQL
    db_url: mysql://root:123456@localhost:3306/test
    # Postgres
    db_url: postgres://postgres:123456@localhost/test
    schemaname: public # only postgres/clickhouse enable
    # Sqlite
    db_url: data.db
    # ClickHouse (HTTP protocol)
    db_url: http://localhost:8123
    schemaname: default # database name for clickhouse
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

## Rust sqlx template example:
    use serde::{Deserialize, Serialize};
    use sqlx::mysql::MySqlRow;
    use sqlx::{FromRow, Row};
    
    {% if table.comment -%}
    	/// {{ table.comment }}
    {% endif -%}
    {% for index in table.index_key -%}
        /// 索引：{{index}}
    {% endfor -%}
    
    
    #[derive(Serialize, Deserialize, PartialEq, Clone)]
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
    
    
    impl<'c> FromRow<'c, MySqlRow<'c>> for {{ table.struct_name }} {
        fn from_row(row: &MySqlRow) -> Result<Self, sqlx::Error> {
            Ok({{ table.struct_name }} {
    {%- for v in table.fields %}
                {{ v.field_name }}: row.get( {{ loop.index0 }} ),
    {%- endfor %}        
            })
        }
    }

## Rust template example:
    /*
    use serde::{Deserialize, Serialize};
    use sqlx::FromRow;
    use error::Result;
    use super::postgres_pool::POSTGRES_POOL;
    
    pub const TABLE_NAME: &str = "{{table.table_name}}";
    
    pub const FIELDS: &str = "{%- for field in table.fields -%}{{field.field_name}}{%- if loop.last == false -%},{%- endif -%}{%- endfor -%}";
    
    {% if table.comment -%}
        /// {{ table.comment }}
    {% endif -%}
    {% for index in table.unique_key -%}
        /// Unique：{{index}}
    {% endfor -%}
    {% for index in table.index_key -%}
        /// Indexes：{{index}}
    {% endfor -%}
    #[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
    pub struct {{ table.struct_name }} {
    {%- for v in table.fields %}
        {% if v.comment -%}
            /// {{ v.comment }} {% if v.database_field_type %} field_type: {{ v.database_field_type }}{% endif %}{% if v.default %} default: {{ v.default }}{% endif %} {% if v.default == '' %} default: ''{% endif %}
        {% endif -%}
        {% if v.is_null == 1 -%}
            {%- if v.field_name == 'type' -%}
                pub r#{{ v.field_name }}: Option<{{ v.field_type }}>,
            {%- else -%}
                pub {{ v.field_name }}: Option<{{ v.field_type }}>,
            {%- endif -%}
        {%- else -%}
            {% if v.field_type == 'time::OffsetDateTime' -%}
                #[serde(with = "time::serde::rfc3339::option", default)]
                pub {{ v.field_name }}: Option<{{ v.field_type }}>,
            {% elif v.field_type == 'chrono::NaiveDateTime' -%}
                pub {{ v.field_name }}: Option<{{ v.field_type }}>,
            {%- else -%}
                {%- if v.field_name == 'type' -%}
                    pub r#{{ v.field_name }}: {{ v.field_type }},
                {%- else -%}
                    pub {{ v.field_name }}: {{ v.field_type }},
                {%- endif -%}
            {%- endif -%}
        {%- endif -%}
    {%- endfor %}
    }
    */
    
    /*
    impl {{table.struct_name}} {
    /*
        pub async fn insert(&self) -> Result<u64> {
            let sql = format!("INSERT INTO {{table.table_name}} ({}) VALUES({% for field in table.fields -%}${{loop.index}}{% if loop.last == false %},{% endif %}{%- endfor %})", FIELDS);
            let mut pool = POSTGRES_POOL.acquire().await?;
            let data = sqlx::query(&sql)
            {%- for field in table.fields %}
                {% if field.field_name == 'type' -%}
                    .bind(&self.r#{{field.field_name}})
                {%- else -%}
                    .bind(&self.{{field.field_name}})
                {%- endif %}
            {%- endfor %}
            .execute(&mut *pool)
            .await?
            .last_insert_id();
            Ok(data)
        }
    */

    /*
    pub async fn select_all() -> Result<Vec<Self>> {
        let sql = format!("SELECT {} from {} {% for v in table.fields -%}{%- if v.field_name == 'is_deleted' -%} WHERE is_deleted = 0 {%- endif -%}{%- endfor -%}", FIELDS, TABLE_NAME);
        let mut pool = POSTGRES_POOL.acquire().await?;
        let data = sqlx::query_as::<_, Self>(&sql).fetch_all(&mut *pool).await?;
        Ok(data)
    }
    */

    {% for indexes in table.unique_key %}
    /*
    pub async fn select_optional_by{%- for index in indexes -%}
    _{{index}}
    {%- endfor %}({%- for index in indexes -%}{{index}}: {%- for v in table.fields -%}{%- if v.field_name == index -%}{%- if v.field_type == 'String' -%}&str{%- else -%}{{v.field_type}}{%- endif -%}{%- endif -%}{%- endfor -%},{%- endfor -%})->Result<Option<Self>>{
        let sql = format!("SELECT {} from {} WHERE {% for index in indexes -%} {%- if loop.last %} {{index}} = ${{loop.index}} {% else %} {{index}} = ${{loop.index}} AND {%- endif -%}{%- endfor -%}{%- for v in table.fields -%}{%- if v.field_name == 'is_deleted' -%} AND is_deleted = 0 {%- endif -%}{%- endfor -%}", FIELDS, TABLE_NAME);
        let mut pool = POSTGRES_POOL.acquire().await?;
        let data = sqlx::query_as::<_, Self>(&sql)
        {% for index in indexes -%}
            {%- for v in table.fields -%}{%- if v.field_name == index -%}{%- if v.field_type == 'String' -%}.bind({{index}}){%- else -%}.bind({{index}}){%- endif -%}{%- endif -%}{%- endfor -%}
        {% endfor -%}
        .fetch_optional(&mut *pool)
        .await?;
        Ok(data)
    }
    */
    {% endfor -%}
    
    {% for indexes in table.unique_key %}
    /*
    pub async fn select_one_by {%- for index in indexes -%}
    _{{index}}
    {%- endfor %}({%- for index in indexes -%}{{index}}: {%- for v in table.fields -%}{%- if v.field_name == index -%}{%- if v.field_type == 'String' -%}&str{%- else -%}{{v.field_type}}{%- endif -%}{%- endif -%}{%- endfor -%},{%- endfor -%})->Result<Self>{
        let sql = format!("SELECT {} from {} WHERE {% for index in indexes -%} {%- if loop.last %} {{index}} = ${{loop.index}} {% else %} {{index}} = ? AND {%- endif -%}{%- endfor -%}{%- for v in table.fields -%}{%- if v.field_name == 'is_deleted' -%} AND is_deleted = 0 {%- endif -%}{%- endfor -%}", FIELDS, TABLE_NAME);
        let mut pool = POSTGRES_POOL.acquire().await?;
        let data = sqlx::query_as::<_, Self>(&sql)
        {% for index in indexes -%}
            {%- for v in table.fields -%}{%- if v.field_name == index -%}{%- if v.field_type == 'String' -%}.bind({{index}}){%- else -%}.bind({{index}}){%- endif -%}{%- endif -%}{%- endfor -%}
        {% endfor -%}
        .fetch_one(&mut *pool)
        .await?;
        Ok(data)
    }
    */
    {% endfor -%}
    
    {% for indexes in table.index_key %}
    /*
    pub async fn select_many_by{%- for index in indexes -%}
    _{{index}}
    {%- endfor -%}_by_page({%- for index in indexes -%}{{index}}: {%- for v in table.fields -%}{%- if v.field_name == index -%}{%- if v.field_type == 'String' -%}&str{%- else -%}{{v.field_type}}{%- endif -%}{%- endif -%}{%- endfor -%},{%- endfor -%}page_no: u64, page_size: u64)->Result<Vec<Self>>{
        let sql = format!("SELECT {} from {} WHERE {% for index in indexes -%} {%- if loop.last %} {{index}} = ? {% else %} {{index}} = ${{loop.index}} AND {%- endif -%}{%- endfor -%} {%- for v in table.fields -%}{%- if v.field_name == 'is_deleted' -%} AND is_deleted = 0 {%- endif -%}{%- endfor %} limit {},{}", FIELDS, TABLE_NAME, (page_no - 1) * page_size, page_no * page_size);
        let mut pool = POSTGRES_POOL.acquire().await?;
        let data = sqlx::query_as::<_, Self>(&sql)
        {% for index in indexes -%}
            {%- for v in table.fields -%}{%- if v.field_name == index -%}{%- if v.field_type == 'String' -%}.bind({{index}}){%- else -%}.bind({{index}}){%- endif -%}{%- endif -%}{%- endfor -%}
        {% endfor -%}
        .fetch_all(&mut *pool)
        .await?;
        Ok(data)
    }
    */
    {% endfor -%}
    
    {% for indexes in table.index_key %}
    /*
    pub async fn select_all_by{%- for index in indexes -%}
    _{{index}}
    {%- endfor -%}({%- for index in indexes -%}{{index}}: {%- for v in table.fields -%}{%- if v.field_name == index -%}{%- if v.field_type == 'String' -%}&str{%- else -%}{{v.field_type}}{%- endif -%}{%- endif -%}{%- endfor -%},{%- endfor -%})->Result<Vec<Self>>{
        let sql = format!("SELECT {} from {} WHERE {% for index in indexes -%} {%- if loop.last %} {{index}} = ${{loop.index}} {% else %} {{index}} = ? AND {%- endif -%}{%- endfor -%} {%- for v in table.fields -%}{%- if v.field_name == 'is_deleted' -%} AND is_deleted = 0 {%- endif -%}{%- endfor -%}", FIELDS, TABLE_NAME);
        let mut pool = POSTGRES_POOL.acquire().await?;
        let data = sqlx::query_as::<_, Self>(&sql)
        {% for index in indexes -%}
            {%- for v in table.fields -%}{%- if v.field_name == index -%}{%- if v.field_type == 'String' -%}.bind({{index}}){%- else -%}.bind({{index}}){%- endif -%}{%- endif -%}{%- endfor -%}
        {% endfor -%}
        .fetch_all(&mut *pool)
        .await?;
        Ok(data)
    }
    */
    {% endfor -%}
    
    {%- for v in table.fields -%}
        {%- if v.field_name == 'is_deleted' -%}
    {% for indexes in table.unique_key %}
    /*
    pub async fn delete_one_by {%- for index in indexes -%}
    _{{index}}
    {%- endfor %}({%- for index in indexes -%}{{index}}: {%- for v in table.fields -%}{%- if v.field_name == index -%}{%- if v.field_type == 'String' -%}&str{%- else -%}{{v.field_type}}{%- endif -%}{%- endif -%}{%- endfor -%},{%- endfor -%})->Result<u64>{
        let sql = format!("UPDATE {} SET is_deleted = 1 WHERE {% for index in indexes -%} {%- if loop.last %} {{index}} = ${{loop.index}} {% else %} {{index}} = ? AND {%- endif -%}{%- endfor -%}{%- for v in table.fields -%}{%- if v.field_name == 'is_deleted' -%} AND is_deleted = 0 {%- endif -%}{%- endfor -%}", TABLE_NAME);
        let mut pool = POSTGRES_POOL.acquire().await?;
        let data = sqlx::query(&sql)
        {% for index in indexes -%}
            {%- for v in table.fields -%}{%- if v.field_name == index -%}{%- if v.field_type == 'String' -%}.bind({{index}}){%- else -%}.bind({{index}}){%- endif -%}{%- endif -%}{%- endfor -%}
        {% endfor -%}
        .execute(&mut *pool)
        .await?.rows_affected();
        Ok(data)
    }
    */
    {% endfor -%}
    
    {% for indexes in table.index_key %}
    /*
    pub async fn delete_many_by {%- for index in indexes -%}
    _{{index}}
    {%- endfor %}({%- for index in indexes -%}{{index}}: {%- for v in table.fields -%}{%- if v.field_name == index -%}{%- if v.field_type == 'String' -%}&str{%- else -%}{{v.field_type}}{%- endif -%}{%- endif -%}{%- endfor -%},{%- endfor -%})->Result<u64>{
        let sql = format!("UPDATE {} SET is_deleted = 1 WHERE {% for index in indexes -%} {%- if loop.last %} {{index}} = ${{loop.index}} {% else %} {{index}} = ${{loop.index}} AND {%- endif -%}{%- endfor -%}{%- for v in table.fields -%}{%- if v.field_name == 'is_deleted' -%} AND is_deleted = 0 {%- endif -%}{%- endfor -%}", TABLE_NAME);
        let mut pool = POSTGRES_POOL.acquire().await?;
        let data = sqlx::query(&sql)
        {% for index in indexes -%}
            {%- for v in table.fields -%}{%- if v.field_name == index -%}{%- if v.field_type == 'String' -%}.bind({{index}}){%- else -%}.bind({{index}}){%- endif -%}{%- endif -%}{%- endfor -%}
        {% endfor -%}
        .execute(&mut *pool)
        .await?.rows_affected();
        Ok(data)
    }
    */
    {% endfor -%}
    {%- endif -%}
    {%- endfor -%}
    }
    */

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
