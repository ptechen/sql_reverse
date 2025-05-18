use std::sync::{LazyLock, RwLock};

pub static POSTGRES_TEMPLATE: LazyLock<RwLock<&str>> = LazyLock::new(|| {
    RwLock::new(
        r#"
/*
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use error::Result;
use super::POSTGRES_POOL;

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
    pub async fn select_optional_by {%- for index in indexes -%}
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
    "#,
    )
});
