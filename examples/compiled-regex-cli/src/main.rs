use compiled_regex::regex;

regex!(r"a");

fn main() {
    println!("{}", MyRegex::is_match("bbb"));
}
