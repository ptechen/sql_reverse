use app_arguments::{ApplicationArguments, Command};
mod app_arguments;
mod error;
mod reverse_struct;
mod table;
mod template;

use crate::error::Result;
use crate::reverse_struct::export::export;
use crate::reverse_struct::gen_struct::GenStruct;
use crate::reverse_struct::mysql_impl::MysqlStruct;
use crate::reverse_struct::postgres_impl::PostgresStruct;
use crate::reverse_struct::sqlite_impl::SqliteImpl;
use crate::table::Table;
use crate::template::kit::Kit;
use crate::template::render::{Render, TemplateType};
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<()> {
    let args = ApplicationArguments::from_args();
    let key = args.command;
    match key {
        Command::Mysql(opt) => {
            let config = MysqlStruct::load(&opt.file).await?;
            let mysql = MysqlStruct::init(config).await?;
            let tables = mysql.run(&opt.custom_field_type).await?;
            Table::check_download_tera(&opt.template_path, &opt.template_name, TemplateType::Mysql)
                .await?;
            Table::render_rust(
                &opt.template_path,
                &opt.template_name,
                &opt.suffix,
                &mysql.config.output_dir,
                &tables,
            )
            .await?;
        }

        Command::Postgres(opt) => {
            let config = PostgresStruct::load(&opt.file).await?;
            let postgres = PostgresStruct::init(config).await?;
            let tables = postgres.run(&opt.custom_field_type).await?;
            Table::check_download_tera(
                &opt.template_path,
                &opt.template_name,
                TemplateType::Postgres,
            )
            .await?;
            Table::render_rust(
                &opt.template_path,
                &opt.template_name,
                &opt.suffix,
                &postgres.config.output_dir,
                &tables,
            )
            .await?;
        }
        Command::Sqlite(opt) => {
            let config = SqliteImpl::load(&opt.file).await?;
            let sqlite = SqliteImpl::init(config).await?;
            let tables = sqlite.run(&opt.custom_field_type).await?;
            Table::check_download_tera(
                &opt.template_path,
                &opt.template_name,
                TemplateType::Sqlite,
            )
            .await?;
            Table::render_rust(
                &opt.template_path,
                &opt.template_name,
                &opt.suffix,
                &sqlite.config.output_dir,
                &tables,
            )
            .await?;
        }
        Command::Export => {
            export().await?;
        }
    }
    Ok(())
}
