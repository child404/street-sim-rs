use criterion::{black_box, criterion_group, criterion_main, Criterion};
use text_matcher_rs::StreetMatcher;

fn bench_street_matcher(c: &mut Criterion) {
    c.bench_function("sm with dir", |b| {
        b.iter(|| {
            StreetMatcher::new(black_box(None), black_box(None))
                .match_by_plz(black_box("qu du seujet 36"), black_box(Some(1201)))
        })
    });
    c.bench_function("sm without dir ", |b| {
        b.iter(|| {
            StreetMatcher::new(black_box(None), black_box(None))
                .match_by_plz(black_box("qu du seujet 36"), black_box(None))
        })
    });
    c.bench_function("sm without dir missed first letter", |b| {
        b.iter(|| {
            StreetMatcher::new(black_box(None), black_box(None))
                .match_by_plz(black_box("u du seujet 36"), black_box(None))
        })
    });
}

criterion_group!(benches, bench_street_matcher);
criterion_main!(benches);
