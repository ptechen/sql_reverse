use app_arguments::{ApplicationArguments, Command};
mod app_arguments;
use sql_reverse_error::result::Result;
use sql_reverse_struct::gen_struct::GenStruct;
use sql_reverse_struct::mysql_struct;
use sql_reverse_struct::mysql_struct::MysqlStruct;
use sql_reverse_struct::postgres_struct;
use sql_reverse_struct::postgres_struct::PostgresStruct;
use sql_reverse_template::render::Render;
use sql_reverse_template::table::Table;
use structopt::StructOpt;
use tracing::Level;
use sql_reverse_struct::export::export;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    let args = ApplicationArguments::from_args();
    let key = args.command;
    match key {
        Command::Mysql(opt) => {
            let config = mysql_struct::MysqlStruct::load(&opt.file).await?;
            let mysql = MysqlStruct::new(config)?;
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
            let config = postgres_struct::PostgresStruct::load(&opt.file).await?;
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
        },
        Command::Export => {
            export().await?;
        }
    }
    Ok(())
}
