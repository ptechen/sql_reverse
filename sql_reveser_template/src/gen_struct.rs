use crate::table::Table;
use async_trait::async_trait;
use sql_reveser_error::result::Result;

#[async_trait]
pub trait GenStruct {
    async fn run(&self) -> Result<Vec<Table>>;

    async fn get_rust_type(&self, field_type: &str) -> Result<String>;

    /// 字符串首字母大写
    async fn first_char_to_uppercase(&self, params: &str) -> Result<String> {
        let mut v: Vec<char> = params.chars().collect();
        v[0] = v[0].to_uppercase().nth(0).unwrap();
        let res = v.into_iter().collect();
        Ok(res)
    }
}
