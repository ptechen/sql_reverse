use tokio::fs::create_dir;
use tokio::io::AsyncWriteExt;

const FLAG: &str = "// ***************************************以下是自定义代码区域******************************************";
const FLAG2: &str = r#"
/*
example: [
    {"skip_fields": ["updated_at", "created_at"], "filename": "table_name1"},
    {"contain_fields": ["updated_at", "created_at"], "filename": "table_name2"}
]
*/
// *************************************************************************************************"#;

use crate::error::Result;
use crate::table::Table;
use crate::template::mysql::MYSQL_TEMPLATE;
use crate::template::postgres::POSTGRES_TEMPLATE;
use crate::template::sqlite::SQLITE_TEMPLATE;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tera::{Context, Tera};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FilterFields {
    pub skip_fields: Option<Vec<String>>,
    pub contain_fields: Option<Vec<String>>,
    pub filename: String,
}

pub enum TemplateType {
    Mysql,
    Sqlite,
    Postgres,
}

async fn filter_fields(table: &Table, params: Vec<FilterFields>) -> Result<Vec<(Table, String)>> {
    let mut list = vec![];
    for field in params.iter() {
        if field.skip_fields.is_some() {
            let data = table
                .skip_fields(field.skip_fields.to_owned().unwrap())
                .await;
            list.push((data, field.filename.to_owned()));
        } else if field.contain_fields.is_some() {
            let data = table
                .contain_fields(field.contain_fields.to_owned().unwrap())
                .await;
            list.push((data, field.filename.to_owned()));
        }
    }
    Ok(list)
}

pub trait Render {
    async fn check_download_tera(
        template_path: &str,
        template_name: &str,
        template_type: TemplateType,
    ) -> Result<()> {
        let file = format!("{}{}", template_path.replace("*", ""), template_name);
        let _ = tokio::fs::create_dir(template_path.replace("*", "")).await;
        if !tokio::fs::try_exists(&file).await.unwrap_or_default() {
            let mut fs = tokio::fs::File::options()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&file)
                .await?;
            match template_type {
                TemplateType::Mysql => {
                    let data = MYSQL_TEMPLATE.read().unwrap().as_bytes();
                    fs.write_all(data).await?;
                }
                TemplateType::Sqlite => {
                    let data = SQLITE_TEMPLATE.read().unwrap().as_bytes();
                    fs.write_all(data).await?;
                }
                TemplateType::Postgres => {
                    let data = POSTGRES_TEMPLATE.read().unwrap().as_bytes();
                    fs.write_all(data).await?;
                }
            }
        }
        Ok(())
    }
    async fn render_rust(
        template_path: &str,
        template_name: &str,
        suffix: &str,
        output_dir: &str,
        tables: &Vec<Table>,
    ) -> Result<()> {
        let _ = create_dir(output_dir).await;
        let tera = Tera::new(template_path)?;
        let mut mods = vec![];
        for table in tables {
            mods.push(format!("pub mod {};\n", table.table_name));
            let (mut struct_str, mut custom, filepath) = Self::render_table(
                &tera,
                table,
                template_name,
                suffix,
                output_dir,
                &table.table_name,
            )
            .await?;
            if !custom.is_empty() {
                let data: Vec<&str> = custom.split("*/").collect();
                let data = data.first().unwrap_or(&"").to_string();
                let data = data.replace("/*", "");
                let data = data.trim();
                let params: Vec<FilterFields> = serde_json::from_str(data).unwrap_or(vec![]);
                let filters = filter_fields(table, params).await?;
                for filter in filters.iter() {
                    mods.push(format!("pub mod {};\n", filter.1));
                    let (mut struct_str, custom, filepath) = Self::render_table(
                        &tera,
                        &filter.0,
                        template_name,
                        suffix,
                        output_dir,
                        &filter.1,
                    )
                    .await?;
                    struct_str = struct_str + "\n" + FLAG + custom.as_str();
                    Self::write_to_file(&filepath, &struct_str).await?;
                }
            }
            if custom.trim() == "" {
                custom = FLAG2.to_owned();
            }
            struct_str = struct_str + "\n" + FLAG + custom.as_str();
            Self::write_to_file(&filepath, &struct_str).await?;
        }

        if suffix == "rs" {
            let mod_path = format!("{}/mod.{}", output_dir, suffix);
            Self::append_to_file(mods, &mod_path).await?;
        }

        Ok(())
    }

    async fn render_table(
        tera: &Tera,
        table: &Table,
        template_name: &str,
        suffix: &str,
        output_dir: &str,
        filename: &str,
    ) -> Result<(String, String, String)> {
        let mut context = Context::new();
        context.insert("table", table);
        let struct_str = tera.render(template_name, &context)?;
        let filepath = format!("{}/{}.{}", output_dir, filename, suffix);
        let content = tokio::fs::read_to_string(&filepath)
            .await
            .unwrap_or_default();
        let vv: Vec<&str> = content.split(FLAG).collect();
        let custom = vv.get(1).unwrap_or(&"").to_string();
        Ok((struct_str, custom, filepath))
    }

    async fn write_to_file(filepath: &str, content: &str) -> Result<()> {
        let filepath = Path::new(&filepath);
        let mut f = tokio::fs::File::options()
            .create(true)
            .truncate(true)
            .write(true)
            .open(filepath)
            .await?;
        f.write_all(content.as_bytes()).await?;
        Ok(())
    }

    async fn append_to_file(mods: Vec<String>, filepath: &str) -> Result<()> {

            if let Ok(mut fs) = tokio::fs::File::options().create_new(true).write(true).open(filepath).await{
            fs.write_all(r#"
//pub static MYSQL_POOL:std::sync::LazyLock<sqlx::mysql::MySqlPool> = std::sync::LazyLock::new(|| {
//    sqlx::mysql::MySqlPool::connect_lazy("mysql://root:123456@127.0.0.1:3306/test").expect("connect mysql error")
//});

//pub static POSTGRES_POOL:std::sync::LazyLock<sqlx::postgres::PgPool> = std::sync::LazyLock::new(|| {
//    sqlx::postgres::PgPool::connect_lazy("postgres://postgres:123456@127.0.0.1:5432/test").expect("connect postgres error")
//});

//pub static SQLITE_POOL:std::sync::LazyLock<sqlx::sqlite::SqlitePool> = std::sync::LazyLock::new(|| {
//    sqlx::sqlite::SqlitePool::connect_lazy("test.db??mode=rwc").expect("connect sqlite error")
//});
"#.as_bytes()).await?;
        }
        let file_content = tokio::fs::read_to_string(filepath)
            .await
            .unwrap_or_default();
        for v in mods.iter() {
            if !file_content.contains(v) {
                let mut file = tokio::fs::File::options()
                    .append(true)
                    .create(true)
                    .open(filepath)
                    .await?;
                file.write_all(v.as_bytes()).await?;
            };
        }
        Ok(())
    }
}
