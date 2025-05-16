use crate::error::result::Result;
use crate::reverse_struct::common::CustomConfig;
use regex::Regex;
use std::collections::BTreeMap;
use std::io::{self, Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub trait Kit {
    /// 字符串首字母大写
    fn first_char_to_uppercase(params: &str) -> String {
        let mut v: Vec<char> = params.chars().collect();
        v[0] = v[0].to_uppercase().nth(0).unwrap();
        let res = v.into_iter().collect();
        res
    }

    fn get_field_type(
        field_type: &str,
        field_name: &str,
        field_type_map: &BTreeMap<String, String>,
    ) -> Result<String> {
        for (k, v) in field_type_map.iter() {
            let r = Regex::new(k.trim())?;
            if r.is_match(&field_type) {
                return Ok(v.to_string());
            }
        }
        Self::write_red(&format!(
            "field_name:{}, {} field type does not match, default type [String] will be used",
            field_name, field_type
        ))?;
        Ok(String::from("String"))
    }

    fn write_red(text: &str) -> io::Result<()> {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
        writeln!(&mut stdout, "{}", text)
    }

    async fn load(filename: &str) -> Result<CustomConfig> {
        let s = tokio::fs::read_to_string(filename).await?;
        let config: CustomConfig = serde_yaml::from_str(&s)?;
        Ok(config)
    }
}
