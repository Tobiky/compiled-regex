pub use compiled_regex_macro::parse_regex;
pub use compiled_regex_core::types;

#[macro_export]
macro_rules! regex {
    // Only for testing purposes, should not be in final product
    ($regex:literal) => {
        #[macro_use] compiled_regex::parse_regex!($regex);
    };
    ($name:ident = $regex:literal) => {
        #[macro_use] type $name = compiled_regex::parse_regex!($regex);
    };
    ($name:ident : $regex:literal) => {
        #[macro_use] compiled_regex::regex!($name = $regex);
    };
}
