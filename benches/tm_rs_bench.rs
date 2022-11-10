use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::PathBuf;
use text_matcher_rs::{MatchAlgo, StreetMatcher, SwissStreet, TextMatcher};

fn bench_street_matcher(c: &mut Criterion) {
    c.bench_function("TextMatcher cfind", |b| {
        b.iter(|| {
            let street = SwissStreet::new("ch de saint-cierges 3,fas23dfsfsdf").value;
            let mat = TextMatcher::cfind_matches_in_file(
                black_box(0.6),
                black_box(100),
                black_box(&street),
                black_box(&PathBuf::from("./test_data/street_names.txt")),
                black_box(Some(8)),
                black_box(MatchAlgo::JaroWinkler),
            )
            .unwrap();
            let matcher = TextMatcher::new(
                black_box(0.6),
                black_box(1),
                black_box(MatchAlgo::Levenshtein),
            );
            matcher.find_matches_from(black_box(&mat), black_box(&street));
        })
    });
    c.bench_function("TextMatcher find", |b| {
        b.iter(|| {
            TextMatcher::new(black_box(0.5), black_box(500), black_box(MatchAlgo::Jaro))
                .find_matches_in_file(
                    black_box("ch de saint-cierges 3"),
                    black_box(&PathBuf::from("./test_data/street_names.txt")),
                    black_box(None),
                )
        })
    });
    c.bench_function("StreetMatcher by place with dir", |b| {
        b.iter(|| {
            StreetMatcher::new(black_box(None), black_box(None)).match_by_place(
                black_box("ch de saint-cierges 3"),
                black_box(Some("bercher")),
            )
        })
    });
    c.bench_function("StreetMatcher with dir", |b| {
        b.iter(|| {
            StreetMatcher::new(black_box(None), black_box(None))
                .match_by_plz(black_box("qu du seujet 36"), black_box(Some(1201)))
        })
    });
    c.bench_function("StreetMatcher without dir ", |b| {
        b.iter(|| {
            StreetMatcher::new(black_box(None), black_box(None))
                .match_by_plz(black_box("qu du seujet 36"), black_box(None))
        })
    });
    c.bench_function("StreetMatcher without dir missed first letter", |b| {
        b.iter(|| {
            StreetMatcher::new(black_box(None), black_box(None))
                .match_by_plz(black_box("u du seujet 36"), black_box(None))
        })
    });
}

criterion_group!(benches, bench_street_matcher);
criterion_main!(benches);
