use criterion::{criterion_group, criterion_main, Criterion, Fun};
use stable_bloom_filter::buckets::Buckets;

fn bench(c: &mut Criterion) {
    let increment = Fun::new("Increment", |b, _| {
        let mut buckets = Buckets::new(100, 8);
        b.iter(|| {
            for i in 0..500 {
                buckets.increment(i % 100, 1);
            }
        })
    });

    let set = Fun::new("Set", |b, _| {
        let mut buckets = Buckets::new(100, 8);
        b.iter(|| {
            for i in 0..500 {
                buckets.set(i % 100, 1);
            }
        })
    });

    let get = Fun::new("Get", |b, _| {
        let mut buckets = Buckets::new(100, 8);

        for i in 0..500 {
            buckets.set(i % 100, 1);
        }

        b.iter(|| {
            for i in 0..500 {
                buckets.get(i % 100);
            }
        })
    });

    let functions = vec![increment, set, get];
    c.bench_functions("Buckets", functions, 0);
}

criterion_group!(benches, bench);
criterion_main!(benches);
