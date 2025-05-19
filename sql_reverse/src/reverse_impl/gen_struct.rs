use crate::error::Result;
use crate::table::{Table, Table2Comment};
use std::collections::BTreeMap;

pub trait GenStruct {
    async fn run(&self, filename: &str) -> Result<Vec<Table>> {
        let tables = self.get_tables().await?;
        let fields_type = self.load_custom_fields_type(filename).await?;
        self.update_type_fields(fields_type).await;
        let templates = self.gen_templates(tables).await?;
        Ok(templates)
    }

    async fn load_custom_fields_type(
        &self,
        filename: &str,
    ) -> Result<Option<BTreeMap<String, String>>> {
        if filename.is_empty() {
            return Ok(None);
        }
        let s = tokio::fs::read_to_string(filename).await?;
        let fields_type: BTreeMap<String, String> = serde_json::from_str(&s)?;
        Ok(Some(fields_type))
    }

    async fn get_tables(&self) -> Result<Vec<Table2Comment>>;
    async fn update_type_fields(&self, map: Option<BTreeMap<String, String>>);
    async fn gen_templates(&self, tables: Vec<Table2Comment>) -> Result<Vec<Table>>;

    async fn filter_tables(
        &self,
        tables: &mut Vec<Table2Comment>,
        include_tables: &Option<Vec<String>>,
        exclude_tables: &Option<Vec<String>>,
    ) {
        if let Some(include_tables) = include_tables {
            *tables = tables
                .iter()
                .filter_map(|table| {
                    if include_tables.contains(&table.table_name) {
                        Some(table.to_owned())
                    } else {
                        None
                    }
                })
                .collect();
        };
        if let Some(exclude_tables) = exclude_tables {
            *tables = tables
                .iter()
                .filter_map(|table| {
                    if exclude_tables.contains(&table.table_name) {
                        None
                    } else {
                        Some(table.to_owned())
                    }
                })
                .collect()
        }
    }

    async fn index_key(&self, table_name: &str) -> Result<(Vec<Vec<String>>, Vec<Vec<String>>)>;
}
