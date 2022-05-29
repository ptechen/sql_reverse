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
    /// Export default database field types
    #[structopt(name = "export")]
    Export,
}

#[derive(Debug, StructOpt)]
pub struct Sql {
    /// Input config file to read
    #[structopt(short = "f", default_value = "./reverse.yml")]
    pub file: String,
    /// Input template path
    #[structopt(short = "p", default_value = "templates/*")]
    pub template_path: String,
    /// Input template name
    #[structopt(short = "n", default_value = "base.tera")]
    pub template_name: String,
    /// Custom field type
    #[structopt(short = "c", default_value = "./default.json")]
    pub custom_field_type: String,
    /// Suffix of the generated file
    #[structopt(short = "s", default_value = "rs")]
    pub suffix: String,
}
