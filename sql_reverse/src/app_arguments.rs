use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "classify")]
pub struct ApplicationArguments {
    #[structopt(subcommand)]
    pub command: Command,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    /// Mysql OPTIONS
    #[structopt(name = "mysql")]
    Mysql(Sql),
    /// PostgresSQL OPTIONS
    #[structopt(name = "postgres")]
    Postgres(Sql),
    /// Sqlite OPTIONS
    #[structopt(name = "sqlite")]
    Sqlite(Sql),
    /// Export default database field types
    #[structopt(name = "export")]
    Export,
}

#[derive(Debug, StructOpt)]
pub struct Sql {
    /// Input database config file to read, example: -f ./reverse.yml
    #[structopt(short = "f", default_value = "./reverse.yml")]
    pub file: String,
    /// Input template path example: -p 'templates/*'
    #[structopt(short = "p", default_value = "templates/*")]
    pub template_path: String,
    /// Input template name, example: -n base.tera
    #[structopt(short = "n", default_value = "base.tera")]
    pub template_name: String,
    /// Custom field type, example: -c ./default.json
    #[structopt(short = "c", default_value = "")]
    pub custom_field_type: String,
    /// Suffix of the generated file, example: -s rs
    #[structopt(short = "s", default_value = "rs")]
    pub suffix: String,
}
