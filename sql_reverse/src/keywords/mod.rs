use std::sync::LazyLock;
use tokio::sync::OnceCell;

mod rust;

pub struct Default;

impl Keywords for Default {}

pub trait Keywords {
    fn check_field_name(&self, field_name: &str) -> String {
        field_name.to_string()
    }
}

pub static LANGUAGE: LazyLock<Box<dyn Keywords + Send + Sync>> =
    LazyLock::new(|| match LANGUAGE_ONCE.get().unwrap().as_str() {
        "rs" => Box::new(rust::Rust),
        _ => Box::new(Default),
    });

pub static LANGUAGE_ONCE: OnceCell<String> = OnceCell::const_new();

pub async fn get_or_init(language: &str) -> String {
    LANGUAGE_ONCE
        .get_or_init(|| async move { language.to_string() })
        .await
        .clone()
}
