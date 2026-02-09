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
    Mysql(Mysql),
    /// Postgres OPTIONS
    #[structopt(name = "postgres")]
    Postgres(Postgres),
    /// Sqlite OPTIONS
    #[structopt(name = "sqlite")]
    Sqlite(Sqlite),
    /// Clickhouse OPTIONS
    #[structopt(name = "clickhouse")]
    Clickhouse(Clickhouse),
    /// TDengine OPTIONS
    #[structopt(name = "tdengine")]
    Tdengine(Tdengine),
    /// Export default database field types
    #[structopt(name = "export")]
    Export,
}

#[derive(Debug, StructOpt)]
pub struct Mysql {
    /// Input database config file to read, example: -f ./reverse.yml
    #[structopt(short = "f", default_value = "./reverse.yml")]
    pub file: String,
    /// Input template path example: -p 'templates/*'
    #[structopt(short = "p", default_value = "templates/*")]
    pub template_path: String,
    /// Input template name, example: -n mysql.tera
    #[structopt(short = "n", default_value = "mysql.tera")]
    pub template_name: String,
    /// Custom field type, example: -c ./default.json
    #[structopt(short = "c", default_value = "")]
    pub custom_field_type: String,
    /// Suffix of the generated file, example: -s rs
    #[structopt(short = "s", default_value = "rs")]
    pub suffix: String,
}

#[derive(Debug, StructOpt)]
pub struct Postgres {
    /// Input database config file to read, example: -f ./reverse.yml
    #[structopt(short = "f", default_value = "./reverse.yml")]
    pub file: String,
    /// Input template path example: -p 'templates/*'
    #[structopt(short = "p", default_value = "templates/*")]
    pub template_path: String,
    /// Input template name, example: -n postgres.tera
    #[structopt(short = "n", default_value = "postgres.tera")]
    pub template_name: String,
    /// Custom field type, example: -c ./default.json
    #[structopt(short = "c", default_value = "")]
    pub custom_field_type: String,
    /// Suffix of the generated file, example: -s rs
    #[structopt(short = "s", default_value = "rs")]
    pub suffix: String,
}

#[derive(Debug, StructOpt)]
pub struct Sqlite {
    /// Input database config file to read, example: -f ./reverse.yml
    #[structopt(short = "f", default_value = "./reverse.yml")]
    pub file: String,
    /// Input template path example: -p 'templates/*'
    #[structopt(short = "p", default_value = "templates/*")]
    pub template_path: String,
    /// Input template name, example: -n sqlite.tera
    #[structopt(short = "n", default_value = "sqlite.tera")]
    pub template_name: String,
    /// Custom field type, example: -c ./default.json
    #[structopt(short = "c", default_value = "")]
    pub custom_field_type: String,
    /// Suffix of the generated file, example: -s rs
    #[structopt(short = "s", default_value = "rs")]
    pub suffix: String,
}

#[derive(Debug, StructOpt)]
pub struct Clickhouse {
    /// Input database config file to read, example: -f ./reverse.yml
    #[structopt(short = "f", default_value = "./reverse.yml")]
    pub file: String,
    /// Input template path example: -p 'templates/*'
    #[structopt(short = "p", default_value = "templates/*")]
    pub template_path: String,
    /// Input template name, example: -n clickhouse.tera
    #[structopt(short = "n", default_value = "clickhouse.tera")]
    pub template_name: String,
    /// Custom field type, example: -c ./default.json
    #[structopt(short = "c", default_value = "")]
    pub custom_field_type: String,
    /// Suffix of the generated file, example: -s rs
    #[structopt(short = "s", default_value = "rs")]
    pub suffix: String,
}

#[derive(Debug, StructOpt)]
pub struct Tdengine {
    /// Input database config file to read, example: -f ./reverse.yml
    #[structopt(short = "f", default_value = "./reverse.yml")]
    pub file: String,
    /// Input template path example: -p 'templates/*'
    #[structopt(short = "p", default_value = "templates/*")]
    pub template_path: String,
    /// Input template name, example: -n tdengine.tera
    #[structopt(short = "n", default_value = "tdengine.tera")]
    pub template_name: String,
    /// Custom field type, example: -c ./default.json
    #[structopt(short = "c", default_value = "")]
    pub custom_field_type: String,
    /// Suffix of the generated file, example: -s rs
    #[structopt(short = "s", default_value = "rs")]
    pub suffix: String,
}
