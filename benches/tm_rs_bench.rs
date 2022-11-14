use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::PathBuf;
use text_matcher_rs::*;

fn bench_street_matcher(c: &mut Criterion) {
    c.bench_function("Place constructor", |b| {
        b.iter(|| {
            Place::new(black_box("Bern City"));
        })
    });

    c.bench_function("StreetMatcher new approach", |b| {
        b.iter(|| {
            street_sim::find_matches::<Plz>(
                black_box(&Street::new("Mühlematt 7-11 (Emmenbrücke)", None).unwrap()),
                black_box(None),
                black_box(None),
            );
        })
    });

    c.bench_function("TextMatcher cfind", |b| {
        b.iter(|| {
            let cfg = Config::new(Sens::new(0.6), 100, SimAlgo::JaroWinkler, None);
            text_sim::fast_cmp_with_file(
                black_box(
                    &Street::new("ch de saint-cierges 3,fas23dfsfsdf", None)
                        .unwrap()
                        .value,
                ),
                black_box(&PathBuf::from("./test_data/street_names.txt")),
                black_box(&cfg),
            )
            .unwrap()
        })
    });
    c.bench_function("TextMatcher find", |b| {
        b.iter(|| {
            let cfg = Config::new(Sens::new(0.5), 500, SimAlgo::Jaro, None);
            text_sim::cmp_with_file(
                black_box(&Text::new("ch de saint-cierges 3")),
                black_box(&PathBuf::from("./test_data/street_names.txt")),
                black_box(&cfg),
            )
        })
    });
    c.bench_function("StreetMatcher by place with dir", |b| {
        b.iter(|| {
            street_sim::find_matches(
                black_box(&Street::new("ch de saint-cierges 3", None).unwrap()),
                black_box(Some(Place::new("bercher"))),
                black_box(None),
            )
        })
    });
    c.bench_function("StreetMatcher with dir", |b| {
        b.iter(|| {
            street_sim::find_matches(
                black_box(&Street::new("qu du seujet 36", None).unwrap()),
                black_box(Some(Plz::new(1201))),
                black_box(None),
            )
        })
    });
    c.bench_function("StreetMatcher without dir ", |b| {
        b.iter(|| {
            street_sim::find_matches::<Plz>(
                black_box(&Street::new("qu du seujet 36", None).unwrap()),
                black_box(None),
                black_box(None),
            )
        })
    });
    c.bench_function("StreetMatcher without dir missed first letter", |b| {
        b.iter(|| {
            street_sim::find_matches::<Plz>(
                black_box(&Street::new("uai du seujet 36", None).unwrap()),
                black_box(None),
                black_box(None),
            )
        })
    });
}

criterion_group!(benches, bench_street_matcher);
criterion_main!(benches);
