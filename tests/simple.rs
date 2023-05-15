use compiled_regex::parse_regex;
use regex::Regex;

parse_regex!(RaQ4a4 = "^a?a?a?a?aaaa");

#[test]
#[allow(non_snake_case)]
fn aQ4a4() {
    let r = Regex::new("^a?a?a?a?aaaa").unwrap();

    assert_eq!(RaQ4a4::is_match(""), r.is_match(""));
    assert_eq!(RaQ4a4::is_match("a"), r.is_match("a"));
    assert_eq!(RaQ4a4::is_match("aa"), r.is_match("aa"));
    assert_eq!(RaQ4a4::is_match("aaa"), r.is_match("aaa"));
    assert_eq!(RaQ4a4::is_match("aaaa"), r.is_match("aaaa"));
    assert_eq!(RaQ4a4::is_match("aaaaa"), r.is_match("aaaaa"));
    assert_eq!(RaQ4a4::is_match("aaaaaa"), r.is_match("aaaaaa"));
    assert_eq!(RaQ4a4::is_match("aaaaaaa"), r.is_match("aaaaaaa"));
}

parse_regex!(Rbfg = "bfg[34]000");

#[test]
fn bfg() {
    let r = Regex::new("bfg[34]000").unwrap();

    assert_eq!(Rbfg::is_match("bfg3000"), r.is_match("bfg3000"));
    assert_eq!(Rbfg::is_match("bfg4000"), r.is_match("bfg4000"));
    assert_eq!(Rbfg::is_match("bfg400"), r.is_match("bfg400"));
    assert_eq!(Rbfg::is_match("bfg2000"), r.is_match("bfg2000"));
    assert_eq!(Rbfg::is_match("bfg3"), r.is_match("bfg3"));
    assert_eq!(Rbfg::is_match("bfg4"), r.is_match("bfg4"));
    assert_eq!(Rbfg::is_match("bag3000"), r.is_match("bag3000"));
}

parse_regex!(Rkthlund = "kth|lund");

#[test]
fn kthlund() {
    let r = Regex::new("kth|lund").unwrap();

    assert_eq!(Rkthlund::is_match("kth"), r.is_match("kth"));
    assert_eq!(Rkthlund::is_match("lund"), r.is_match("lund"));
    assert_eq!(Rkthlund::is_match("ktha"), r.is_match("ktha"));
    assert_eq!(Rkthlund::is_match("lunda"), r.is_match("lunda"));
}
