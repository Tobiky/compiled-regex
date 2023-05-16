use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};

use regex::Regex;
use compiled_regex::parse_regex;

// max 'a' repeat string size
// 16 is longest anyhow
const N: usize = 32;

// n = 1
pub fn compiled_n1(c: &mut Criterion) {
    parse_regex!(Rgx = "^a?a");

    let mut group = c.benchmark_group("continues compiled ^a?^1a^1");

    for text in (1..N).into_iter().map(|n| "a".repeat(n)) {
        let text_length = text.chars().count();
        group.throughput(Throughput::Elements(text_length as u64));
        group.bench_with_input(BenchmarkId::from_parameter(text_length), &&text, |b, txt| {
            b.iter(|| Rgx::is_match(txt));
        });
    }
}

pub fn interpreted_n1(c: &mut Criterion) {
    let regex = Regex::new("^a?a").unwrap();

    let mut group = c.benchmark_group("continues interpreted ^a?^1a^1");

    for text in (1..N).into_iter().map(|n| "a".repeat(n)) {
        let text_length = text.chars().count();
        group.throughput(Throughput::Elements(text_length as u64));
        group.bench_with_input(BenchmarkId::from_parameter(text_length), &&text, |b, txt| {
            b.iter(|| regex.is_match(txt));
        });
    }
}

// n = 4
pub fn compiled_n4(c: &mut Criterion) {
    parse_regex!(Rgx = "^a?a?a?a?aaaa");

    let mut group = c.benchmark_group("continues compiled ^a?^4a^4");

    for text in (1..N).into_iter().map(|n| "a".repeat(n)) {
        let text_length = text.chars().count();
        group.throughput(Throughput::Elements(text_length as u64));
        group.bench_with_input(BenchmarkId::from_parameter(text_length), &&text, |b, txt| {
            b.iter(|| Rgx::is_match(txt));
        });
    }
}

pub fn interpreted_n4(c: &mut Criterion) {
    let regex = Regex::new("^a?a?a?a?aaaa").unwrap();

    let mut group = c.benchmark_group("continues interpreted ^a?^4a^4");

    for text in (1..N).into_iter().map(|n| "a".repeat(n)) {
        let text_length = text.chars().count();
        group.throughput(Throughput::Elements(text_length as u64));
        group.bench_with_input(BenchmarkId::from_parameter(text_length), &&text, |b, txt| {
            b.iter(|| regex.is_match(txt));
        });
    }
}

// n = 16
pub fn compiled_n16(c: &mut Criterion) {
    parse_regex!(Rgx = "^a?a?a?a?a?a?a?a?a?a?a?a?a?a?a?a?aaaaaaaaaaaaaaaa");

    let mut group = c.benchmark_group("continues compiled ^a?^16a^16");

    for text in (1..N).into_iter().map(|n| "a".repeat(n)) {
        let text_length = text.chars().count();
        group.throughput(Throughput::Elements(text_length as u64));
        group.bench_with_input(BenchmarkId::from_parameter(text_length), &&text, |b, txt| {
            b.iter(|| Rgx::is_match(txt));
        });
    }
}

pub fn interpreted_n16(c: &mut Criterion) {
    let regex = Regex::new("^a?a?a?a?a?a?a?a?a?a?a?a?a?a?a?a?aaaaaaaaaaaaaaaa").unwrap();

    let mut group = c.benchmark_group("continues interpreted ^a?^16a^16");

    for text in (1..N).into_iter().map(|n| "a".repeat(n)) {
        let text_length = text.chars().count();
        group.throughput(Throughput::Elements(text_length as u64));
        group.bench_with_input(BenchmarkId::from_parameter(text_length), &&text, |b, txt| {
            b.iter(|| regex.is_match(txt));
        });
    }
}



criterion_group!(benches,
    compiled_n1,
    interpreted_n1,
    compiled_n4,
    interpreted_n4,
    compiled_n16,
    interpreted_n16);

criterion_main!(benches);
