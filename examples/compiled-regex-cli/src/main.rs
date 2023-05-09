#[allow(unused_imports)]
use compiled_regex::parse_regex as regex;
#[allow(unused_imports)]
use compiled_regex::parse_regex_output;

use regex::Regex;

regex!(Reg = "a{1,3}b");

fn main() {
    // parse_regex_output!(MyRegex = "a{1,3}b");
    let r = Regex::new("a{1,3}b").unwrap();

    println!("{:?}", Reg::is_match("aaaab"));
    println!("{:?}", r.is_match("aaaab"));
}
