use std::fmt::Display;
use std::sync::{LazyLock, RwLock};

pub static TEMPLATE_TYPE: LazyLock<RwLock<TemplateType>> =
    LazyLock::new(|| RwLock::new(TemplateType::Mysql));

pub fn update_template_type(template_type: TemplateType) {
    *TEMPLATE_TYPE.write().unwrap() = template_type;
}

pub enum TemplateType {
    Mysql,
    Sqlite,
    Postgres,
    Clickhouse,
}

impl Display for TemplateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplateType::Mysql => write!(
                f,
                r#"
pub static MYSQL_POOL:std::sync::LazyLock<sqlx::mysql::MySqlPool> = std::sync::LazyLock::new(|| {{
    sqlx::mysql::MySqlPool::connect_lazy("mysql://root:123456@127.0.0.1:3306/test").expect("connect mysql error")
}});
pub type Result<T> = std::result::Result<T, sqlx::Error>;
"#
            ),
            TemplateType::Postgres => write!(
                f,
                r#"
pub static POSTGRES_POOL:std::sync::LazyLock<sqlx::postgres::PgPool> = std::sync::LazyLock::new(|| {{
    sqlx::postgres::PgPool::connect_lazy("postgres://postgres:123456@127.0.0.1:5432/test").expect("connect postgres error")
}});
pub type Result<T> = std::result::Result<T, sqlx::Error>;
"#
            ),
            TemplateType::Sqlite => write!(
                f,
                r#"
pub static SQLITE_POOL:std::sync::LazyLock<sqlx::sqlite::SqlitePool> = std::sync::LazyLock::new(|| {{
    sqlx::sqlite::SqlitePool::connect_lazy("test.db?mode=rwc").expect("connect sqlite error")
}});
pub type Result<T> = std::result::Result<T, sqlx::Error>;
"#
            ),
            TemplateType::Clickhouse => write!(
                f,
                r#"
pub static CLICKHOUSE_CLIENT:std::sync::LazyLock<clickhouse::Client> = std::sync::LazyLock::new(|| {{
    clickhouse::Client::default().with_url("http://127.0.0.1:8123").with_database("default")
}});
pub type Result<T> = std::result::Result<T, clickhouse::error::Error>;
"#
            ),
        }
    }
}
