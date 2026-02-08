use app_arguments::{ApplicationArguments, Command};
mod app_arguments;
mod error;
mod keywords;
mod reverse_impl;
mod table;
mod template;

use crate::error::Result;
use crate::keywords::get_or_init;
use crate::reverse_impl::export::export;
use crate::reverse_impl::gen_struct::GenStruct;
use crate::reverse_impl::clickhouse_impl::ClickhouseImpl;
use crate::reverse_impl::mysql_impl::MysqlImpl;
use crate::reverse_impl::postgres_impl::PostgresImpl;
use crate::reverse_impl::sqlite_impl::SqliteImpl;
use crate::table::Table;
use crate::template::kit::Kit;
use crate::template::render::Render;
use crate::template::template_type::{TemplateType, update_template_type};
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<()> {
    let args = ApplicationArguments::from_args();
    let key = args.command;
    match key {
        Command::Mysql(opt) => {
            update_template_type(TemplateType::Mysql);
            get_or_init(&opt.suffix).await;
            let config = MysqlImpl::load(&opt.file).await?;
            let mysql = MysqlImpl::init(config).await?;
            let tables = mysql.run(&opt.custom_field_type).await?;
            Table::check_download_tera(&opt.template_path, &opt.template_name).await?;
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
            update_template_type(TemplateType::Postgres);
            get_or_init(&opt.suffix).await;
            let config = PostgresImpl::load(&opt.file).await?;
            let postgres = PostgresImpl::init(config).await?;
            let tables = postgres.run(&opt.custom_field_type).await?;
            Table::check_download_tera(&opt.template_path, &opt.template_name).await?;
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
            update_template_type(TemplateType::Sqlite);
            get_or_init(&opt.suffix).await;
            let config = SqliteImpl::load(&opt.file).await?;
            let sqlite = SqliteImpl::init(config).await?;
            let tables = sqlite.run(&opt.custom_field_type).await?;
            Table::check_download_tera(&opt.template_path, &opt.template_name).await?;
            Table::render_rust(
                &opt.template_path,
                &opt.template_name,
                &opt.suffix,
                &sqlite.config.output_dir,
                &tables,
            )
            .await?;
        }
        Command::Clickhouse(opt) => {
            update_template_type(TemplateType::Clickhouse);
            get_or_init(&opt.suffix).await;
            let config = ClickhouseImpl::load(&opt.file).await?;
            let clickhouse = ClickhouseImpl::init(config).await?;
            let tables = clickhouse.run(&opt.custom_field_type).await?;
            Table::check_download_tera(&opt.template_path, &opt.template_name).await?;
            Table::render_rust(
                &opt.template_path,
                &opt.template_name,
                &opt.suffix,
                &clickhouse.config.output_dir,
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
