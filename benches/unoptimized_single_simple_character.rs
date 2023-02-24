use criterion::{criterion_group, criterion_main};

mod _1 {
    use criterion::{Criterion, black_box};

    #[macro_use]
    compiled_regex::regex!("a");

    pub fn simple_a(c: &mut Criterion) {
        c.bench_function(
            "fib 20",
            |b| b.iter(|| fibonacci(black_box(20))));
    }
}

criterion_group!(benches, _1::simple_a);
criterion_main!(benches);
