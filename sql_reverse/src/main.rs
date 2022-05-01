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

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    let args = ApplicationArguments::from_args();
    let key = args.command;
    match key {
        Command::Mysql(opt) => {
            let config = mysql_struct::MysqlStruct::load(&opt.file).await?;
            let mysql = MysqlStruct::new(config)?;
            let tables = mysql.run().await?;
            Table::render_rust(
                &opt.template_path,
                &opt.template_name,
                &mysql.config.output_dir,
                &tables,
            )
            .await?;
        }

        Command::Postgres(opt) => {
            let config = postgres_struct::PostgresStruct::load(&opt.file).await?;
            let postgres = PostgresStruct::new(config).await?;
            let tables = postgres.run().await?;
            Table::render_rust(
                &opt.template_path,
                &opt.template_name,
                &postgres.config.output_dir,
                &tables,
            )
                .await?;
        }
    }
    Ok(())
}
