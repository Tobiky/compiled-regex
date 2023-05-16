use regex::Regex;

use compiled_regex::parse_regex;

const TEST_INPUTS_RAW: &'static str = include_str!("../data/keggle_urldata_urls.txt");

fn get_lines() -> Vec<&'static str> {
    TEST_INPUTS_RAW.split("\n").collect()
}

#[test]
fn trip() {
    parse_regex!(Rgx = "^(:?tripadvisor\\.at|tripadvisor\\.be|tripadvisor\\.ca|tripadvisor\\.ch|tripadvisor\\.cl|tripadvisor\\.cn|tripadvisor\\.co|tripadvisor\\.co\\.id|tripadvisor\\.co\\.il|tripadvisor\\.co\\.kr|tripadvisor\\.co\\.nz|tripadvisor\\.co\\.uk|tripadvisor\\.co\\.za|tripadvisor\\.com|tripadvisor\\.com\\.ar|tripadvisor\\.com\\.au|tripadvisor\\.com\\.br|tripadvisor\\.com\\.eg|tripadvisor\\.com\\.gr|tripadvisor\\.com\\.hk|tripadvisor\\.com\\.mx|tripadvisor\\.com\\.my|tripadvisor\\.com\\.pe|tripadvisor\\.com\\.ph|tripadvisor\\.com\\.sg|tripadvisor\\.com\\.tr|tripadvisor\\.com\\.tw|tripadvisor\\.com\\.ve|tripadvisor\\.com\\.vn|tripadvisor\\.de|tripadvisor\\.dk|tripadvisor\\.es|tripadvisor\\.fr|tripadvisor\\.ie|tripadvisor\\.in|tripadvisor\\.it|tripadvisor\\.jp|tripadvisor\\.nl|tripadvisor\\.pt|tripadvisor\\.ru|tripadvisor\\.se)#?#\\.listItem:-abp-has\\(\\.sponsored_v2\\)");

    let regex = Regex::new("^(:?tripadvisor\\.at|tripadvisor\\.be|tripadvisor\\.ca|tripadvisor\\.ch|tripadvisor\\.cl|tripadvisor\\.cn|tripadvisor\\.co|tripadvisor\\.co\\.id|tripadvisor\\.co\\.il|tripadvisor\\.co\\.kr|tripadvisor\\.co\\.nz|tripadvisor\\.co\\.uk|tripadvisor\\.co\\.za|tripadvisor\\.com|tripadvisor\\.com\\.ar|tripadvisor\\.com\\.au|tripadvisor\\.com\\.br|tripadvisor\\.com\\.eg|tripadvisor\\.com\\.gr|tripadvisor\\.com\\.hk|tripadvisor\\.com\\.mx|tripadvisor\\.com\\.my|tripadvisor\\.com\\.pe|tripadvisor\\.com\\.ph|tripadvisor\\.com\\.sg|tripadvisor\\.com\\.tr|tripadvisor\\.com\\.tw|tripadvisor\\.com\\.ve|tripadvisor\\.com\\.vn|tripadvisor\\.de|tripadvisor\\.dk|tripadvisor\\.es|tripadvisor\\.fr|tripadvisor\\.ie|tripadvisor\\.in|tripadvisor\\.it|tripadvisor\\.jp|tripadvisor\\.nl|tripadvisor\\.pt|tripadvisor\\.ru|tripadvisor\\.se)#?#\\.listItem:-abp-has\\(\\.sponsored_v2\\)").unwrap();

    for (i, line) in get_lines().iter().enumerate() {
        assert_eq!(Rgx::is_match(line), regex.is_match(line), "{i}: {}", line);
    }
}

#[test]
fn online() {
    parse_regex!(Rgx = "^(?:[^:/?#]+:)?(?://(?:[^/?#]*\\.)?)?kissanimeonline\\.com/driectlink");

    let regex = Regex::new("^(?:[^:/?#]+:)?(?://(?:[^/?#]*\\.)?)?kissanimeonline\\.com/driectlink").unwrap();

    for (i, line) in get_lines().iter().enumerate() {
        assert_eq!(Rgx::is_match(line), regex.is_match(line), "{i}: {}", line);
    }
}

#[test]
fn aws() {
    parse_regex!(Rgx = "/(?:[^\\w\\d_\\-\\.%]|\\$)https?:\\/\\/s3\\..*\\..*\\.amazonaws\\.com\\/[a-f0-9]{45,}\\/[a-f,0-9]{8,10}");

    let regex = Regex::new("/(?:[^\\w\\d_\\-\\.%]|\\$)https?:\\/\\/s3\\..*\\..*\\.amazonaws\\.com\\/[a-f0-9]{45,}\\/[a-f,0-9]{8,10}").unwrap();

    for (i, line) in get_lines().iter().enumerate() {
        assert_eq!(Rgx::is_match(line), regex.is_match(line), "{i}: {}", line);
    }
}

