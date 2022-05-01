#[cfg(feature = "async-std")]
use async_std::{
    fs::{OpenOptions, Path},
    prelude::*,
};

#[cfg(feature = "tokio")]
use tokio::fs::OpenOptions;

#[cfg(feature = "tokio")]
use tokio::io::AsyncWriteExt;

const FLAG: &'static str = "// ***************************************以下是自定义代码区域******************************************";

use crate::table::Table;
use async_trait::async_trait;
use quicli::prelude::*;
use sql_reverse_error::result::Result;
use std::path::Path;
use tera::{Context, Tera};

#[async_trait]
pub trait Render {
    async fn render_rust(
        template_path: &str,
        template_name: &str,
        output_dir: &str,
        tables: &Vec<Table>,
    ) -> Result<()> {
        create_dir(output_dir)?;
        let tera = Tera::new(template_path)?;
        let mut context = Context::new();
        let mod_path = format!("{}/mod.rs", output_dir);
        let mut mods = vec![];
        for table in tables {
            mods.push(format!("pub mod {};\n", table.table_name));
            context.insert("template", table);
            let mut struct_str = tera.render(template_name, &context)?;
            let filepath = format!("{}/{}.rs", output_dir, table.table_name);
            let content = read_file(&filepath).unwrap_or_default();
            let vv: Vec<&str> = content.split(FLAG).collect();
            let custom = vv.get(1).unwrap_or(&"").to_string();
            struct_str = struct_str + "\n" + FLAG + custom.as_str();
            async_ok!(Self::write_to_file(&filepath, &struct_str))?;
        }
        async_ok!(Self::append_to_file(mods, &mod_path))?;
        Ok(())
    }

    async fn write_to_file(filepath: &str, content: &str) -> Result<()> {
        let filepath = Path::new(&filepath);
        let s = write_to_file(filepath, content)?;
        Ok(s)
    }

    async fn append_to_file(mods: Vec<String>, filepath: &str) -> Result<()> {
        let file_content = read_file(filepath).unwrap_or_default();
        for v in mods.iter() {
            if !file_content.contains(v) {
                let mut file = async_ok!(OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(filepath))?;
                async_ok!(file.write(v.as_bytes()))?;
            };
        }
        Ok(())
    }
}
