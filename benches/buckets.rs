use criterion::{criterion_group, criterion_main, Criterion, Fun};
use rand::{thread_rng, Rng};
use stable_bloom_filter::buckets::Buckets;

fn bench(c: &mut Criterion) {
    let increment = Fun::new("Increment", |b, _| {
        let mut buckets = Buckets::new(10_000, 8);
        let mut rng = thread_rng();
        let mut data = vec![];

        for _ in 0..2_500_000 {
            let r: usize = rng.gen_range(0, 10_000);
            data.push(r);
        }

        b.iter(|| {
            for i in data.iter() {
                buckets.increment(*i, 1);
            }
        })
    });

    let set = Fun::new("Set", |b, _| {
        let mut buckets = Buckets::new(1000, 8);
        let mut rng = thread_rng();
        let mut data = vec![];

        for _ in 0..100_000 {
            let r: usize = rng.gen_range(0, 1000);
            let v: u8 = rng.gen_range(0, 255);
            data.push((r, v));
        }

        b.iter(|| {
            for (i, v) in data.iter() {
                buckets.set(*i, *v);
            }
        })
    });

    let get = Fun::new("Get", |b, _| {
        let mut buckets = Buckets::new(1000, 8);
        let mut rng = thread_rng();
        let mut data = vec![];

        for i in 0..=1000 {
            let v: u8 = rng.gen_range(0, 255);
            buckets.set(i % 1000, v);
        }

        for _ in 0..100_000 {
            let r: usize = rng.gen_range(0, 1000);
            data.push(r);
        }

        b.iter(|| {
            for i in 0..1000 {
                buckets.get(i);
            }
        })
    });

    let functions = vec![increment, set, get];
    c.bench_functions("Buckets", functions, 0);
}

criterion_group!(benches, bench);
criterion_main!(benches);
