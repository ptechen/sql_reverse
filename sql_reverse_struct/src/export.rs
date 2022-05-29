use sql_reverse_error::result::Result;
use crate::mysql_struct;
use crate::postgres_struct;
use quicli::fs::write_to_file;

pub async fn export() -> Result<()> {
    let mysql_default = mysql_struct::FIELD_TYPE.clone();
    let postgres_default = postgres_struct::FIELD_TYPE.clone();
    let mysql_default = format!("{:#?}", mysql_default);
    let postgres_default = format!("{:#?}", postgres_default);
    write_to_file("./mysql_default.json", &mysql_default.replace(",\n}", "\n}"))?;
    write_to_file("./postgres_default.json", &postgres_default.replace(",\n}", "\n}"))?;
    Ok(())
}
