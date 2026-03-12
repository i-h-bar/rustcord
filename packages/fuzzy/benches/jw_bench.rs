use criterion::{Bencher, Criterion, criterion_group, criterion_main};

use fuzzy;

fn bench(c: &mut Criterion) {
    let sentence_1 = "some Long sentence";
    let sentence_2 = "another long sentence";

    c.bench_function("Bitmask JW", |b: &mut Bencher| {
        b.iter(|| fuzzy::jaro_winkler_ascii_bitmask(&sentence_1, &sentence_2))
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(50);
    targets = bench
}

criterion_main!(benches);
