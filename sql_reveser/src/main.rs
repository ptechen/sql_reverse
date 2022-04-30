use app_arguments::{ApplicationArguments, Command};
use quicli::prelude::*;
mod app_arguments;
use mysql_struct::mysql_struct;
use sql_template::gen_struct::GenStruct;
use structopt::StructOpt;
use sql_template::table::Table;
use sql_template::render::Render;

#[tokio::main]
async fn main() -> CliResult {
    let args = ApplicationArguments::from_args();
    let key = args.command;
    match key {
        Command::Mysql(opt) => {
            let mysql = mysql_struct::MysqlStruct::load(&opt.file).unwrap();
            let tables = mysql.run().await?;
            Table::render_rust(&opt.template_path, &opt.template_name, &mysql.config.output_dir, &tables).await?;
        }
    }
    Ok(())
}
