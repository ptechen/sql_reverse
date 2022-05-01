#[macro_use]
extern crate sql_reveser_error;



pub mod render;
pub mod table;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
