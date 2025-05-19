use crate::keywords::Keywords;
use fn_macro::hashmap;
use std::collections::HashMap;
use std::sync::LazyLock;

pub static RUST_KEYS: LazyLock<HashMap<&str, ()>> = LazyLock::new(|| {
    hashmap!(
        "if" => (),
        "else" => (),
        "loop" => (),
        "while" => (),
        "for" => (),
        "break" => (),
        "continue" => (),
        "return" => (),
        "match" => (),
        "fn" => (),
        "let" => (),
        "mut" => (),
        "const" => (),
        "static" => (),
        "struct" => (),
        "enum" => (),
        "union" =>(),
        "impl" => (),
        "trait" => (),
        "type" => (),
        "use" => (),
        "mod" => (),
        "pub" => (),
        "crate" => (),
        "self" => (),
        "Self" => (),
        "super" => (),
        "dyn" => (),
        "move" => (),
        "ref" => (),
        "async" => (),
        "await" => (),
        "unsafe" => (),
        "as" => (),
    )
});

pub struct Rust;

impl Keywords for Rust {
    fn check_field_name(&self, field_name: &str) -> String {
        if RUST_KEYS.contains_key(field_name) {
            format!("r#{}", field_name)
        } else {
            field_name.to_string()
        }
    }
}
