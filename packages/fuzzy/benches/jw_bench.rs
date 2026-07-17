use criterion::{Bencher, Criterion, criterion_group, criterion_main};

use fuzzy;

fn bench(c: &mut Criterion) {
    let cases = [
        ("JW short (~11)", "black lotus", "black lotos"),
        (
            "JW typical (~18)",
            "the gitrog monster",
            "the gitrog monstre",
        ),
        (
            "JW long (~57)",
            "okina temple to the grandfathers okina temple grandfather",
            "okina temple to the grandfathers okima temple grandfhater",
        ),
    ];

    for (name, s1, s2) in cases {
        let mut group = c.benchmark_group(name);
        group.bench_function("bitmask", |b: &mut Bencher| {
            b.iter(|| fuzzy::jaro_winkler_ascii_bitmask(&s1, &s2))
        });
        group.bench_function("simd", |b: &mut Bencher| {
            b.iter(|| fuzzy::jaro_winkler_ascii_simd(&s1, &s2))
        });
        group.bench_function("strsim", |b: &mut Bencher| {
            b.iter(|| strsim::jaro_winkler(s1, s2))
        });
        group.finish();
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(50);
    targets = bench
}

criterion_main!(benches);
