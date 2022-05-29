use mysql;
use quicli;
use serde_yaml;
use tera;

#[allow(dead_code)]
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    RegexError(regex::Error),
    SerdeYamlError(serde_yaml::Error),
    SerdeJsonError(serde_json::Error),
    IoError(std::io::Error),
    QuiCliError(quicli::prelude::Error),
    TeraError(tera::Error),
    MysqlUrlError(mysql::UrlError),
    MysqlError(mysql::error::Error),
    PostgresError(tokio_postgres::Error),
    CustomError(String),
}

#[derive(Debug, Clone, Default)]
pub struct CustomError {
    pub message: String,
}

impl From<String> for CustomError {
    fn from(message: String) -> CustomError {
        CustomError { message }
    }
}

impl<'a> From<CustomError> for &'a dyn std::error::Error {
    fn from(error: CustomError) -> &'a dyn std::error::Error {
        error.into()
    }
}

impl std::error::Error for CustomError {}

impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Error::RegexError(ref err) => Some(err),
            Error::SerdeYamlError(ref error) => Some(error),
            Error::SerdeJsonError(ref error) => Some(error),
            Error::IoError(ref error) => Some(error),
            Error::QuiCliError(ref error) => {
                let s: CustomError = From::from(error.to_string());
                Some(s.into())
            }
            Error::TeraError(ref error) => Some(error),
            Error::MysqlUrlError(ref error) => Some(error),
            Error::MysqlError(ref error) => Some(error),
            Error::PostgresError(ref error) => Some(error),
            Error::CustomError(ref error) => {
                let s: CustomError = From::from(error.to_string());
                Some(s.into())
            }
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Error::RegexError(ref error) => error.fmt(f),
            Error::SerdeYamlError(ref error) => error.fmt(f),
            Error::SerdeJsonError(ref error) => error.fmt(f),
            Error::IoError(ref error) => error.fmt(f),
            Error::QuiCliError(ref error) => error.fmt(f),
            Error::TeraError(ref error) => error.fmt(f),
            Error::MysqlUrlError(ref error) => error.fmt(f),
            Error::MysqlError(ref error) => error.fmt(f),
            Error::PostgresError(ref error) => error.fmt(f),
            Error::CustomError(ref error) => error.fmt(f),
        }
    }
}

impl From<String> for Error {
    fn from(error: String) -> Self {
        Error::CustomError(error)
    }
}

impl From<mysql::error::Error> for Error {
    fn from(error: mysql::error::Error) -> Self {
        Error::MysqlError(error)
    }
}

impl From<tokio_postgres::Error> for Error {
    fn from(error: tokio_postgres::Error) -> Self {
        Error::PostgresError(error)
    }
}

impl From<mysql::UrlError> for Error {
    fn from(error: mysql::UrlError) -> Self {
        Error::MysqlUrlError(error)
    }
}

impl From<tera::Error> for Error {
    fn from(error: tera::Error) -> Self {
        Error::TeraError(error)
    }
}

impl From<quicli::prelude::Error> for Error {
    fn from(error: quicli::prelude::Error) -> Error {
        Error::CustomError(error.to_string())
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(error: serde_yaml::Error) -> Error {
        Error::SerdeYamlError(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Error {
        Error::SerdeJsonError(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(s: std::io::Error) -> Self {
        Error::IoError(s)
    }
}

impl From<regex::Error> for Error {
    fn from(s: regex::Error) -> Self {
        Error::RegexError(s)
    }
}
