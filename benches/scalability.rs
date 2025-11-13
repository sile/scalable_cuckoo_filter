use criterion::*;
use mimalloc::MiMalloc;

use rand::Rng;
use scalable_cuckoo_filter::ScalableCuckooFilter;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert");

    for precision in [0.1, 0.001, 0.0001, 0.00001] {
        // the benchmarks will run 10-13M iterations
        let mut filter = ScalableCuckooFilter::<u64>::new(1_000_000, precision);
        let mut i = 0;

        group.bench_function(BenchmarkId::new("precision", precision), |b| {
            b.iter(|| {
                filter.insert(&i);
                i += 1;
            })
        });
    }
}

fn contains(c: &mut Criterion) {
    let mut group = c.benchmark_group("contains");

    for precision in [0.1, 0.001, 0.0001, 0.00001] {
        let filter = ScalableCuckooFilter::<u64>::new(1_000_000, precision);

        group.bench_function(BenchmarkId::new("precision", precision), |b| {
            b.iter_batched(
                || {
                    let item: u64 = rand::rng().random();

                    let mut f = filter.clone();
                    f.insert(&item);
                    item
                },
                |item| {
                    filter.contains(&item);
                },
                BatchSize::SmallInput,
            )
        });
    }
}

criterion_group!(benches, insert, contains);
criterion_main!(benches);
