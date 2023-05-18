use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

use regex::Regex;

use compiled_regex::parse_regex;

const TEST_INPUTS_RAW: &'static str = include_str!("../data/keggle_urldata_urls.txt");

fn get_lines() -> Vec<&'static str> {
    TEST_INPUTS_RAW.split("\n").collect()
}

// simple vs. intermediate vs. complex in terms of created automata, not length

pub fn compiled_simple(c: &mut Criterion) {
    let lines = get_lines();

    // tripadvisor.[domain]#?#.listItem:-abp-has.sponsored_v2, mostly just alternation
    parse_regex!(Rgx = "^(:?tripadvisor\\.at|tripadvisor\\.be|tripadvisor\\.ca|tripadvisor\\.ch|tripadvisor\\.cl|tripadvisor\\.cn|tripadvisor\\.co|tripadvisor\\.co\\.id|tripadvisor\\.co\\.il|tripadvisor\\.co\\.kr|tripadvisor\\.co\\.nz|tripadvisor\\.co\\.uk|tripadvisor\\.co\\.za|tripadvisor\\.com|tripadvisor\\.com\\.ar|tripadvisor\\.com\\.au|tripadvisor\\.com\\.br|tripadvisor\\.com\\.eg|tripadvisor\\.com\\.gr|tripadvisor\\.com\\.hk|tripadvisor\\.com\\.mx|tripadvisor\\.com\\.my|tripadvisor\\.com\\.pe|tripadvisor\\.com\\.ph|tripadvisor\\.com\\.sg|tripadvisor\\.com\\.tr|tripadvisor\\.com\\.tw|tripadvisor\\.com\\.ve|tripadvisor\\.com\\.vn|tripadvisor\\.de|tripadvisor\\.dk|tripadvisor\\.es|tripadvisor\\.fr|tripadvisor\\.ie|tripadvisor\\.in|tripadvisor\\.it|tripadvisor\\.jp|tripadvisor\\.nl|tripadvisor\\.pt|tripadvisor\\.ru|tripadvisor\\.se)#?#\\.listItem:-abp-has\\(\\.sponsored_v2\\)");

    c.bench_with_input(BenchmarkId::new("compiled simple", lines.len()), &lines, |b, ls| {
        b.iter(|| for line in ls { Rgx::is_match(line); })
    });
}

pub fn interpreted_simple(c: &mut Criterion) {
    let lines = get_lines();

    // tripadvisor.[domain]#?#.listItem:-abp-has.sponsored_v2, mostly just alternation
    let regex = Regex::new("^(:?tripadvisor\\.at|tripadvisor\\.be|tripadvisor\\.ca|tripadvisor\\.ch|tripadvisor\\.cl|tripadvisor\\.cn|tripadvisor\\.co|tripadvisor\\.co\\.id|tripadvisor\\.co\\.il|tripadvisor\\.co\\.kr|tripadvisor\\.co\\.nz|tripadvisor\\.co\\.uk|tripadvisor\\.co\\.za|tripadvisor\\.com|tripadvisor\\.com\\.ar|tripadvisor\\.com\\.au|tripadvisor\\.com\\.br|tripadvisor\\.com\\.eg|tripadvisor\\.com\\.gr|tripadvisor\\.com\\.hk|tripadvisor\\.com\\.mx|tripadvisor\\.com\\.my|tripadvisor\\.com\\.pe|tripadvisor\\.com\\.ph|tripadvisor\\.com\\.sg|tripadvisor\\.com\\.tr|tripadvisor\\.com\\.tw|tripadvisor\\.com\\.ve|tripadvisor\\.com\\.vn|tripadvisor\\.de|tripadvisor\\.dk|tripadvisor\\.es|tripadvisor\\.fr|tripadvisor\\.ie|tripadvisor\\.in|tripadvisor\\.it|tripadvisor\\.jp|tripadvisor\\.nl|tripadvisor\\.pt|tripadvisor\\.ru|tripadvisor\\.se)#?#\\.listItem:-abp-has\\(\\.sponsored_v2\\)").unwrap();

    c.bench_with_input(BenchmarkId::new("interpreted simple", lines.len()), &lines, |b, ls| {
        b.iter(|| for line in ls { regex.is_match(line); })
    });
}

pub fn compiled_intermediate(c: &mut Criterion) {
    let lines = get_lines();

    // more usage of groups, alternation, and quantification
    parse_regex!(Rgx = "^/(?:[^\\w\\d_\\-\\.%]|\\$)https?:\\/\\/s3\\..*\\..*\\.amazonaws\\.com\\/[a-f0-9]{45,}\\/[a-f,0-9]{8,10}");

    c.bench_with_input(BenchmarkId::new("compiled intermediate", lines.len()), &lines, |b, ls| {
        b.iter(|| for line in ls { Rgx::is_match(line); })
    });
}

pub fn interpreted_intermediate(c: &mut Criterion) {
    let lines = get_lines();

    // more usage of groups, alternation, and quantification
    let regex = Regex::new("^/(?:[^\\w\\d_\\-\\.%]|\\$)https?:\\/\\/s3\\..*\\..*\\.amazonaws\\.com\\/[a-f0-9]{45,}\\/[a-f,0-9]{8,10}").unwrap();

    c.bench_with_input(BenchmarkId::new("interpreted intermediate", lines.len()), &lines, |b, ls| {
        b.iter(|| for line in ls { regex.is_match(line); })
    });
}

pub fn compiled_complex(c: &mut Criterion) {
    let lines = get_lines();

    // heavy usage of groups, alternation, and quantification
    parse_regex!(Rgx = "^(?:[^:/?#]+:)?(?://(?:[^/?#]*\\.)?)?kissanimeonline\\.com/driectlink");

    c.bench_with_input(BenchmarkId::new("compiled complex", lines.len()), &lines, |b, ls| {
        b.iter(|| for line in ls { Rgx::is_match(line); })
    });
}

pub fn interpreted_complex(c: &mut Criterion) {
    let lines = get_lines();

    // heavy usage of groups, alternation, and quantification
    let regex = Regex::new("^(?:[^:/?#]+:)?(?://(?:[^/?#]*\\.)?)?kissanimeonline\\.com/driectlink").unwrap();

    c.bench_with_input(BenchmarkId::new("interpreted complex", lines.len()), &lines, |b, ls| {
        b.iter(|| for line in ls { regex.is_match(line); })
    });
}


criterion_group!(benches,
	compiled_simple,
	interpreted_simple,
	compiled_intermediate,
	interpreted_intermediate,
	compiled_complex,
    interpreted_complex);
criterion_main!(benches);
