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

    // Batch functions over a realistic candidate list (normalised card names).
    let target = "lightnig bolt";
    let candidates = [
        "lightning bolt",
        "lightning strike",
        "chain lightning",
        "lightning helix",
        "wall of lightning",
        "lightning greaves",
        "bolt of keranos",
        "galvanic bolt",
        "black lotus",
        "the gitrog monster",
        "gideon of the trials",
        "okina temple to the grandfathers",
        "counterspell",
        "brainstorm",
        "dark ritual",
        "swords to plowshares",
        "birds of paradise",
        "llanowar elves",
        "sol ring",
        "force of will",
    ];

    let mut group = c.benchmark_group("winkliest");
    group.bench_function("match", |b: &mut Bencher| {
        b.iter(|| fuzzy::winkliest_match(&target, candidates))
    });
    group.bench_function("sort", |b: &mut Bencher| {
        b.iter(|| fuzzy::winkliest_sort(&target, candidates))
    });
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(50);
    targets = bench
}

criterion_main!(benches);
