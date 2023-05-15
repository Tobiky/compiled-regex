use criterion::{black_box, criterion_group, criterion_main, Criterion};

const BENCH_STRING: &'static str = include_str!("../data/alice29.txt");

use regex::Regex;
use compiled_regex::parse_regex;

// n = 1
pub fn compiled_n1(c: &mut Criterion) {
    parse_regex!(Rgx = "^a?a");

    c.bench_function("compiled ^a?^1a^1", |b| b.iter(|| Rgx::is_match(black_box(BENCH_STRING))));
}

pub fn interpreted_n1(c: &mut Criterion) {
    let regex = Regex::new("^a?a").unwrap();

    c.bench_function("interpreted ^a?^1a^1", |b| b.iter(|| regex.is_match(black_box(BENCH_STRING))));
}

// n = 2
pub fn compiled_n2(c: &mut Criterion) {
    parse_regex!(Rgx = "^a?a?aa");

    c.bench_function("compiled ^a?^2a^2", |b| b.iter(|| Rgx::is_match(black_box(BENCH_STRING))));
}

pub fn interpreted_n2(c: &mut Criterion) {
    let regex = Regex::new("^a?a?aa").unwrap();

    c.bench_function("interpreted ^a?^2a^2", |b| b.iter(|| regex.is_match(black_box(BENCH_STRING))));
}

// n = 4
pub fn compiled_n4(c: &mut Criterion) {
    parse_regex!(Rgx = "^a?a?a?a?aaaa");

    c.bench_function("compiled ^a?^4a^4", |b| b.iter(|| Rgx::is_match(black_box(BENCH_STRING))));
}

pub fn interpreted_n4(c: &mut Criterion) {
    let regex = Regex::new("^a?a?a?a?aaaa").unwrap();

    c.bench_function("interpreted ^a?^4a^4", |b| b.iter(|| regex.is_match(black_box(BENCH_STRING))));
}

// n = 8
pub fn compiled_n8(c: &mut Criterion) {
    parse_regex!(Rgx = "^a?a?a?a?a?a?a?a?aaaaaaaa");

    c.bench_function("compiled ^a?^8a^8", |b| b.iter(|| Rgx::is_match(black_box(BENCH_STRING))));
}

pub fn interpreted_n8(c: &mut Criterion) {
    let regex = Regex::new("^a?a?a?a?a?a?a?a?aaaaaaaa").unwrap();

    c.bench_function("interpreted ^a?^8a^8", |b| b.iter(|| regex.is_match(black_box(BENCH_STRING))));
}

// n = 16
pub fn compiled_n16(c: &mut Criterion) {
    parse_regex!(Rgx = "^a?a?a?a?a?a?a?a?a?a?a?a?a?a?a?a?aaaaaaaaaaaaaaaa");

    c.bench_function("compiled ^a?^16a^16", |b| b.iter(|| Rgx::is_match(black_box(BENCH_STRING))));
}

pub fn interpreted_n16(c: &mut Criterion) {
    let regex = Regex::new("^a?a?a?a?a?a?a?a?a?a?a?a?a?a?a?a?aaaaaaaaaaaaaaaa").unwrap();

    c.bench_function("interpreted ^a?^16a^16", |b| b.iter(|| regex.is_match(black_box(BENCH_STRING))));
}


criterion_group!(benches,
    compiled_n1,
    interpreted_n1,
    compiled_n2,
    interpreted_n2,
    compiled_n4,
    interpreted_n4,
    compiled_n8,
    interpreted_n8,
    compiled_n16,
    interpreted_n16);

criterion_main!(benches);
