use criterion::{Bencher, Criterion, criterion_group, criterion_main};

use fuzzy;

fn bench(c: &mut Criterion) {
    let short_1 = "black lotus";
    let short_2 = "black lotos";

    let typical_1 = "the gitrog monster";
    let typical_2 = "the gitrog monstre";

    let long_1 = "okina temple to the grandfathers okina temple grandfather";
    let long_2 = "okina temple to the grandfathers okima temple grandfhater";

    c.bench_function("Bitmask JW short (~11)", |b: &mut Bencher| {
        b.iter(|| fuzzy::jaro_winkler_ascii_bitmask(&short_1, &short_2))
    });

    c.bench_function("Bitmask JW typical (~18)", |b: &mut Bencher| {
        b.iter(|| fuzzy::jaro_winkler_ascii_bitmask(&typical_1, &typical_2))
    });

    c.bench_function("Bitmask JW long (~57)", |b: &mut Bencher| {
        b.iter(|| fuzzy::jaro_winkler_ascii_bitmask(&long_1, &long_2))
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(50);
    targets = bench
}

criterion_main!(benches);
