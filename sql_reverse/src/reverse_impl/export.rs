use crate::error::Result;
use crate::reverse_impl::{mysql_impl, postgres_impl, sqlite_impl};
use tokio::io::AsyncWriteExt;

pub async fn export() -> Result<()> {
    let mysql_default = mysql_impl::FIELD_TYPE.read().unwrap().clone();
    let postgres_default = postgres_impl::FIELD_TYPE.read().unwrap().clone();
    let sqlite_default = sqlite_impl::FIELD_TYPE.read().unwrap().clone();
    let mysql_default = format!("{:#?}", mysql_default);
    let postgres_default = format!("{:#?}", postgres_default);
    let sqlite_default = format!("{:#?}", sqlite_default);
    let mut fs = tokio::fs::File::options()
        .create(true)
        .truncate(true)
        .write(true)
        .open("./default_mysql.json")
        .await?;
    fs.write_all(mysql_default.replace(",\n}", "\n}").as_bytes())
        .await?;
    let mut fs = tokio::fs::File::options()
        .create(true)
        .truncate(true)
        .write(true)
        .open("./default_postgres.json")
        .await?;
    fs.write_all(postgres_default.replace(",\n}", "\n}").as_bytes())
        .await?;

    let mut fs = tokio::fs::File::options()
        .create(true)
        .truncate(true)
        .write(true)
        .open("./default_sqlite.json")
        .await?;
    fs.write_all(sqlite_default.replace(",\n}", "\n}").as_bytes())
        .await?;
    Ok(())
}
