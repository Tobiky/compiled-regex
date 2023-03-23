#[allow(unused_imports)]
use compiled_regex::parse_regex as regex;
#[allow(unused_imports)]
use compiled_regex::parse_regex_output;

// regex!(Regex = "[abc]b");

fn main() {
    parse_regex_output!(MyRegex = "[abc]b|kab");

    // println!("{:?}", Regex::is_match("kb"));
}
