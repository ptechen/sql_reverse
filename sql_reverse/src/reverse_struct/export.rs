use crate::error::result::Result;
use crate::reverse_struct::{mysql_impl, postgres_impl};
use tokio::io::AsyncWriteExt;

pub async fn export() -> Result<()> {
    let mysql_default = mysql_impl::FIELD_TYPE.read().unwrap().clone();
    let postgres_default = postgres_impl::FIELD_TYPE.read().unwrap().clone();
    let mysql_default = format!("{:#?}", mysql_default);
    let postgres_default = format!("{:#?}", postgres_default);
    let mut fs = tokio::fs::File::options()
        .create(true)
        .truncate(true)
        .write(true)
        .open("./mysql_default.json")
        .await?;
    fs.write_all(mysql_default.replace(",\n}", "\n}").as_bytes())
        .await?;
    let mut fs = tokio::fs::File::options()
        .create(true)
        .truncate(true)
        .write(true)
        .open("./postgres_default.json")
        .await?;
    fs.write_all(postgres_default.replace(",\n}", "\n}").as_bytes())
        .await?;
    Ok(())
}
