#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    RegexError(#[from] regex::Error),
    #[error(transparent)]
    SerdeYamlError(#[from] serde_yaml::Error),
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    ClapError(#[from] clap::Error),
    #[error(transparent)]
    TeraError(#[from] tera::Error),
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),
    #[error(transparent)]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
}
