use tokio::fs::create_dir;
use tokio::io::AsyncWriteExt;

const FLAG: &'static str = "// ***************************************以下是自定义代码区域******************************************";
const FLAG2: &'static str = r#"
/*
example: [
    {"skip_fields": ["updated_at", "created_at"], "filename": "table_name1"},
    {"contain_fields": ["updated_at", "created_at"], "filename": "table_name2"}
]
*/
// *************************************************************************************************"#;

use crate::error::result::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tera::{Context, Tera};
use crate::table::Table;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FilterFields {
    pub skip_fields: Option<Vec<String>>,
    pub contain_fields: Option<Vec<String>>,
    pub filename: String,
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
    async fn render_rust(
        template_path: &str,
        template_name: &str,
        suffix: &str,
        output_dir: &str,
        tables: &Vec<Table>,
    ) -> Result<()> {
        let _ = create_dir(output_dir).await;
        println!("开始生成{}文件...", output_dir);
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
            if custom != "" {
                let data: Vec<&str> = custom.split("*/").collect();
                let data = data.get(0).unwrap_or(&"").to_string();
                let data = data.replace("/*", "");
                let data = data.trim();
                let params: Vec<FilterFields> = serde_json::from_str(&data).unwrap_or(vec![]);
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
        context.insert("template", table); // 兼容之前的版本
        context.insert("table", table);
        let struct_str = tera.render(template_name, &context)?;
        let filepath = format!("{}/{}.{}", output_dir, filename, suffix);
        let content = tokio::fs::read_to_string(&filepath).await?;
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
        let file_content = tokio::fs::read_to_string(filepath).await?;
        for v in mods.iter() {
            if !file_content.contains(v) {
                let mut file =
                    tokio::fs::File::options().append(true).create(true).open(filepath).await?;
                file.write(v.as_bytes()).await?;
            };
        }
        Ok(())
    }
}
