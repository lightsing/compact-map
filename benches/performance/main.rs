use crate::helpers::RandomTest;
use compact_map::CompactMap;
use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use rand::SeedableRng;
use std::collections::HashMap;
use std::hash::Hash;

mod helpers;

macro_rules! run_random {
    ($random_group:ident, $key:ty, $value:ty, $size:expr, $runs:expr) => {
        $random_group
            .throughput(criterion::Throughput::Elements($runs))
            .bench_function(
                BenchmarkId::new(
                    format!(
                        "HashMap[{}:{}]",
                        std::any::type_name::<$key>(),
                        std::any::type_name::<$value>()
                    ),
                    $size,
                ),
                |b| {
                    b.iter_batched(
                        || {
                            let rng = rand_xorshift::XorShiftRng::seed_from_u64(42);
                            RandomTest::<
                                rand_xorshift::XorShiftRng,
                                HashMap<$key, $value>,
                                $key,
                                $value,
                            >::new(rng, HashMap::with_capacity($size), $size)
                        },
                        |random_test| {
                            let mut random_test = black_box(random_test);
                            for _ in 0..$runs {
                                black_box(random_test.random_step());
                            }
                        },
                        BatchSize::SmallInput,
                    );
                },
            )
            .bench_function(
                BenchmarkId::new(
                    format!(
                        "CompactMap[{}:{}]",
                        std::any::type_name::<$key>(),
                        std::any::type_name::<$value>()
                    ),
                    $size,
                ),
                |b| {
                    b.iter_batched(
                        || {
                            let rng = rand_xorshift::XorShiftRng::seed_from_u64(42);
                            RandomTest::<
                                rand_xorshift::XorShiftRng,
                                CompactMap<$key, $value, $size>,
                                $key,
                                $value,
                            >::new(rng, CompactMap::new(), $size)
                        },
                        |random_test| {
                            let mut random_test = black_box(random_test);
                            for _ in 0..$runs {
                                black_box(random_test.random_step());
                            }
                        },
                        BatchSize::SmallInput,
                    );
                },
            );
    };
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut random_group = c.benchmark_group("RandomTest");

    run_random!(random_group, u8, u8, 8, 1000);
    run_random!(random_group, u8, u8, 16, 1000);
    run_random!(random_group, u8, u8, 32, 1000);
    run_random!(random_group, u8, u8, 64, 1000);
    run_random!(random_group, u8, u8, 128, 1000);
    run_random!(random_group, u8, u8, 256, 1000);
    run_random!(random_group, u8, u8, 512, 1000);
    run_random!(random_group, u8, u8, 1024, 1000);

    run_random!(random_group, u16, u16, 8, 1000);
    run_random!(random_group, u16, u16, 16, 1000);
    run_random!(random_group, u16, u16, 32, 1000);
    run_random!(random_group, u16, u16, 64, 1000);
    run_random!(random_group, u16, u16, 128, 1000);
    run_random!(random_group, u16, u16, 256, 1000);
    run_random!(random_group, u16, u16, 512, 1000);
    run_random!(random_group, u16, u16, 1024, 1000);

    run_random!(random_group, u32, u32, 8, 1000);
    run_random!(random_group, u32, u32, 16, 1000);
    run_random!(random_group, u32, u32, 32, 1000);
    run_random!(random_group, u32, u32, 64, 1000);
    run_random!(random_group, u32, u32, 128, 1000);
    run_random!(random_group, u32, u32, 256, 1000);
    run_random!(random_group, u32, u32, 512, 1000);
    run_random!(random_group, u32, u32, 1024, 1000);

    run_random!(random_group, u64, u64, 8, 1000);
    run_random!(random_group, u64, u64, 16, 1000);
    run_random!(random_group, u64, u64, 32, 1000);
    run_random!(random_group, u64, u64, 64, 1000);
    run_random!(random_group, u64, u64, 128, 1000);
    run_random!(random_group, u64, u64, 256, 1000);
    run_random!(random_group, u64, u64, 512, 1000);
    run_random!(random_group, u64, u64, 1024, 1000);

    run_random!(random_group, u128, u128, 8, 1000);
    run_random!(random_group, u128, u128, 16, 1000);
    run_random!(random_group, u128, u128, 32, 1000);
    run_random!(random_group, u128, u128, 64, 1000);
    run_random!(random_group, u128, u128, 128, 1000);
    run_random!(random_group, u128, u128, 256, 1000);
    run_random!(random_group, u128, u128, 512, 1000);
    run_random!(random_group, u128, u128, 1024, 1000);

    random_group.finish()
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
