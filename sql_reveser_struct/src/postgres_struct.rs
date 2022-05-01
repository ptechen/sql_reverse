use crate::common::CustomConfig;
use async_trait::async_trait;
use quicli::prelude::read_file;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sql_reveser_error::result::Result;
use sql_reveser_template::gen_struct::GenStruct;
use sql_reveser_template::table::Table;
use tokio_postgres::{Client, Error, NoTls};

#[derive(Default, Debug, Deserialize, Serialize, Clone)]
pub struct PostgresStruct {
    pub config: CustomConfig,
}

pub struct GenTemplateData {
    pub table_name: String,
    pub struct_name: String,
    // pub mysql_rows: Option<Vec<Row>>,
    pub table_comment: String,
}

impl PostgresStruct {
    pub fn new(config: CustomConfig) -> Result<Self> {
        Ok(Self { config })
    }

    pub fn load(filename: &str) -> Result<Self> {
        let s = read_file(filename)?;
        let config: CustomConfig = serde_yaml::from_str(&s)?;
        Ok(Self { config })
    }
}

#[async_trait]
impl GenStruct for PostgresStruct {
    async fn run(&self) -> Result<Vec<Table>> {
        // Connect to the database.
        let (client, connection) = tokio_postgres::connect(
            &format!(
                "host={} port={} user={} password={} dbname={}",
                self.config.host,
                self.config.port,
                self.config.username,
                self.config.password,
                self.config.database
            ),
            NoTls,
        )
        .await.unwrap();

        // The connection object performs the actual communication with the database,
        // so spawn it off to run on its own.
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        // Now we can execute a simple statement that just returns its parameter.
        let rows = client.query(&format!(
            "select  * from pg_tables where schemaname = '{}'",
            self.config.schemaname.to_owned().unwrap_or_default()
        ), &[]).await.unwrap();
        Ok(vec![])
    }

    async fn get_rust_type(&self, field_type: &str) -> Result<String> {
        // for (k, v) in FIELD_TYPE.iter() {
        //     let r = Regex::new(k.trim()).unwrap();
        //     if r.is_match(&field_type) {
        //         return Ok(v.to_string());
        //     }
        // }
        Ok(String::new())
    }
}
