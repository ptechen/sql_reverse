use std::sync::{LazyLock, RwLock};

pub static CLICKHOUSE_TEMPLATE: LazyLock<RwLock<&str>> = LazyLock::new(|| {
    RwLock::new(
        r#"
use serde::{Deserialize, Serialize};
use clickhouse::Row;
use super::Result;
use super::CLICKHOUSE_CLIENT;

pub const TABLE_NAME: &str = "{{table.table_name}}";

pub const FIELDS: &str = "{%- for field in table.fields -%}{{field.field_name}}{%- if loop.last == false -%},{%- endif -%}{%- endfor -%}";

{% if table.comment -%}
	/// {{ table.comment }}
{% endif -%}
{% for index in table.unique_key -%}
    /// PrimaryKey：{{index}}
{% endfor -%}
{% for index in table.index_key -%}
    /// SortingKey：{{index}}
{% endfor -%}
#[derive(Debug, Clone, Serialize, Deserialize, Row)]
pub struct {{ table.struct_name }} {
{%- for v in table.fields %}
	{% if v.comment -%}
	    /// {{ v.comment }} {% if v.database_field_type %} field_type: {{ v.database_field_type }}{% endif %}{% if v.default %} default: {{ v.default }}{% endif %} {% if v.default == '' %} default: ''{% endif %}
	{% endif -%}
	{% if v.is_null == 1 -%}
    	pub {{ v.field_name }}: Option<{{ v.field_type }}>,
    {%- else -%}
        {% if v.field_type == 'chrono::NaiveDateTime' -%}
    pub {{ v.field_name }}: Option<{{ v.field_type }}>,
        {%- else -%}
            pub {{ v.field_name }}: {{ v.field_type }},
        {%- endif -%}
    {%- endif -%}
{%- endfor %}
}

impl {{table.struct_name}} {
    pub async fn insert(&self) -> Result<()> {
    	let mut insert = CLICKHOUSE_CLIENT.insert(TABLE_NAME)?;
    	insert.write(self).await?;
    	insert.end().await?;
        Ok(())
    }

    pub async fn select_all() -> Result<Vec<Self>> {
        let sql = format!("SELECT {FIELDS} FROM {TABLE_NAME} {% for v in table.fields -%}{%- if v.field_name == 'is_deleted' -%} WHERE is_deleted = 0 {%- endif -%}{%- endfor -%}");
        let data = CLICKHOUSE_CLIENT.query(&sql).fetch_all::<Self>().await?;
        Ok(data)
    }

{% for indexes in table.unique_key %}

    pub async fn select_optional_by {%- for index in indexes -%}
                        _{{index}}
    {%- endfor %}({%- for index in indexes -%}{{index}}: {%- for v in table.fields -%}{%- if v.field_name == index -%}{%- if v.field_type == 'String' -%}&str{%- else -%}{{v.field_type}}{%- endif -%}{%- endif -%}{%- endfor -%},{%- endfor -%})->Result<Option<Self>>{
        let sql = format!("SELECT {FIELDS} FROM {TABLE_NAME} WHERE {% for index in indexes -%} {%- if loop.last %} {{index}} = ? {% else %} {{index}} = ? AND {%- endif -%}{%- endfor -%}{%- for v in table.fields -%}{%- if v.field_name == 'is_deleted' -%} AND is_deleted = 0 {%- endif -%}{%- endfor -%} LIMIT 1");
        let data = CLICKHOUSE_CLIENT.query(&sql)
            {% for index in indexes -%}
                .bind({{index}})
            {% endfor -%}
            .fetch_optional::<Self>()
            .await?;
        Ok(data)
    }

{% endfor -%}

{% for indexes in table.unique_key %}

    pub async fn select_one_by {%- for index in indexes -%}
                        _{{index}}
    {%- endfor %}({%- for index in indexes -%}{{index}}: {%- for v in table.fields -%}{%- if v.field_name == index -%}{%- if v.field_type == 'String' -%}&str{%- else -%}{{v.field_type}}{%- endif -%}{%- endif -%}{%- endfor -%},{%- endfor -%})->Result<Self>{
        let sql = format!("SELECT {FIELDS} FROM {TABLE_NAME} WHERE {% for index in indexes -%} {%- if loop.last %} {{index}} = ? {% else %} {{index}} = ? AND {%- endif -%}{%- endfor -%}{%- for v in table.fields -%}{%- if v.field_name == 'is_deleted' -%} AND is_deleted = 0 {%- endif -%}{%- endfor -%} LIMIT 1");
        let data = CLICKHOUSE_CLIENT.query(&sql)
            {% for index in indexes -%}
                .bind({{index}})
            {% endfor -%}
            .fetch_one::<Self>()
            .await?;
        Ok(data)
    }

{% endfor -%}

{% for indexes in table.index_key %}

    pub async fn select_many_by{%- for index in indexes -%}
                        _{{index}}
    {%- endfor -%}_by_page({%- for index in indexes -%}{{index}}: {%- for v in table.fields -%}{%- if v.field_name == index -%}{%- if v.field_type == 'String' -%}&str{%- else -%}{{v.field_type}}{%- endif -%}{%- endif -%}{%- endfor -%},{%- endfor -%}page_no: u64, page_size: u64)->Result<Vec<Self>>{
        let sql = format!("SELECT {FIELDS} FROM {TABLE_NAME} WHERE {% for index in indexes -%} {%- if loop.last %} {{index}} = ? {% else %} {{index}} = ? AND {%- endif -%}{%- endfor -%} {%- for v in table.fields -%}{%- if v.field_name == 'is_deleted' -%} AND is_deleted = 0 {%- endif -%}{%- endfor %} LIMIT {} OFFSET {}", page_size, (page_no - 1) * page_size);
        let data = CLICKHOUSE_CLIENT.query(&sql)
            {% for index in indexes -%}
                .bind({{index}})
            {% endfor -%}
            .fetch_all::<Self>()
            .await?;
        Ok(data)
    }
{% endfor -%}

{% for indexes in table.index_key %}

    pub async fn select_all_by{%- for index in indexes -%}
                        _{{index}}
    {%- endfor -%}({%- for index in indexes -%}{{index}}: {%- for v in table.fields -%}{%- if v.field_name == index -%}{%- if v.field_type == 'String' -%}&str{%- else -%}{{v.field_type}}{%- endif -%}{%- endif -%}{%- endfor -%},{%- endfor -%})->Result<Vec<Self>>{
        let sql = format!("SELECT {FIELDS} FROM {TABLE_NAME} WHERE {% for index in indexes -%} {%- if loop.last %} {{index}} = ? {% else %} {{index}} = ? AND {%- endif -%}{%- endfor -%} {%- for v in table.fields -%}{%- if v.field_name == 'is_deleted' -%} AND is_deleted = 0 {%- endif -%}{%- endfor -%}");
        let data = CLICKHOUSE_CLIENT.query(&sql)
            {% for index in indexes -%}
                .bind({{index}})
            {% endfor -%}
            .fetch_all::<Self>()
            .await?;
        Ok(data)
    }
{% endfor -%}

{%- for v in table.fields -%}
    {%- if v.field_name == 'is_deleted' -%}
{% for indexes in table.unique_key %}

    pub async fn delete_one_by {%- for index in indexes -%}
                        _{{index}}
    {%- endfor %}({%- for index in indexes -%}{{index}}: {%- for v in table.fields -%}{%- if v.field_name == index -%}{%- if v.field_type == 'String' -%}&str{%- else -%}{{v.field_type}}{%- endif -%}{%- endif -%}{%- endfor -%},{%- endfor -%})->Result<()>{
        let sql = format!("ALTER TABLE {TABLE_NAME} UPDATE is_deleted = 1 WHERE {% for index in indexes -%} {%- if loop.last %} {{index}} = ? {% else %} {{index}} = ? AND {%- endif -%}{%- endfor -%}{%- for v in table.fields -%}{%- if v.field_name == 'is_deleted' -%} AND is_deleted = 0 {%- endif -%}{%- endfor -%}");
        CLICKHOUSE_CLIENT.query(&sql)
            {% for index in indexes -%}
                .bind({{index}})
            {% endfor -%}
            .execute()
            .await?;
        Ok(())
    }
{% endfor -%}

{% for indexes in table.index_key %}

    pub async fn delete_many_by {%- for index in indexes -%}
                        _{{index}}
    {%- endfor %}({%- for index in indexes -%}{{index}}: {%- for v in table.fields -%}{%- if v.field_name == index -%}{%- if v.field_type == 'String' -%}&str{%- else -%}{{v.field_type}}{%- endif -%}{%- endif -%}{%- endfor -%},{%- endfor -%})->Result<()>{
        let sql = format!("ALTER TABLE {TABLE_NAME} UPDATE is_deleted = 1 WHERE {% for index in indexes -%} {%- if loop.last %} {{index}} = ? {% else %} {{index}} = ? AND {%- endif -%}{%- endfor -%}{%- for v in table.fields -%}{%- if v.field_name == 'is_deleted' -%} AND is_deleted = 0 {%- endif -%}{%- endfor -%}");
        CLICKHOUSE_CLIENT.query(&sql)
            {% for index in indexes -%}
                .bind({{index}})
            {% endfor -%}
            .execute()
            .await?;
        Ok(())
    }
{% endfor -%}
    {%- endif -%}
{%- endfor -%}
}
    "#,
    )
});
