use app_arguments::{ApplicationArguments, Command};
mod app_arguments;
mod error;
mod reverse_struct;
mod template;
mod table;

use crate::error::result::Result;
use crate::reverse_struct::export::export;
use crate::reverse_struct::gen_struct::GenStruct;
use crate::reverse_struct::mysql_impl::MysqlStruct;
use crate::reverse_struct::postgres_impl::PostgresStruct;
use crate::template::kit::Kit;
use crate::template::render::Render;
use structopt::StructOpt;
use crate::table::Table;

#[tokio::main]
async fn main() -> Result<()> {
    let args = ApplicationArguments::from_args();
    let key = args.command;
    match key {
        Command::Mysql(opt) => {
            let config = MysqlStruct::load(&opt.file).await?;
            let mysql = MysqlStruct::init(config).await?;
            let tables = mysql.run(&opt.custom_field_type).await?;
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
            let postgres = PostgresStruct::new(config).await?;
            let tables = postgres.run(&opt.custom_field_type).await?;
            Table::render_rust(
                &opt.template_path,
                &opt.template_name,
                &opt.suffix,
                &postgres.config.output_dir,
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
