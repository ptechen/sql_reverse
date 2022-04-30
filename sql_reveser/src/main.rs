use app_arguments::{ApplicationArguments, Command};
mod app_arguments;
use sql_reveser_struct::mysql_struct;
use sql_reveser_template::gen_struct::GenStruct;
use sql_reveser_template::render::Render;
use sql_reveser_template::table::Table;
use sql_reveser_error::result::Result;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<()> {
    let args = ApplicationArguments::from_args();
    let key = args.command;
    match key {
        Command::Mysql(opt) => {
            let mysql = mysql_struct::MysqlStruct::load(&opt.file).unwrap();
            let tables = mysql.run().await?;
            Table::render_rust(
                &opt.template_path,
                &opt.template_name,
                &mysql.config.output_dir,
                &tables,
            )
            .await?;
        }
    }
    Ok(())
}
