#[macro_use]
extern crate criterion;

#[cfg(any(feature = "rocksdb"))]
use std::fs::remove_dir_all;
use std::path::PathBuf;

use criterion::{BenchmarkId, Criterion, Throughput};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use starling::Array;

const KEY_LEN: usize = 32;

#[cfg(not(any(feature = "rocksdb")))]
use starling::hash_tree::HashTree;
#[cfg(feature = "rocksdb")]
use starling::rocks_tree::RocksTree;

#[cfg(not(any(feature = "rocksdb")))]
type Tree = HashTree<KEY_LEN, Vec<u8>>;

#[cfg(feature = "rocksdb")]
type Tree = RocksTree<[u8; KEY_LEN], Vec<u8>>;

/** Benchmarks 1000, 2000, 5000, 10000 inserts to a tree with no previous state */
fn hash_tree_empty_tree_insert_big_benchmark(c: &mut Criterion) {
    let path = PathBuf::from("db");
    let seed = [0xBBu8; 32];
    let mut rng: StdRng = SeedableRng::from_seed(seed);
    let mut group = c.benchmark_group("Big Empty Tree");
    let sizes = vec![1000, 2000, 5000, 10000];
    for size in sizes {
        let kvs = prepare_inserts(size, &mut rng);
        let mut bmt = Tree::open(&path, 160).unwrap();
        let mut keys = kvs.0.clone();
        let values = &kvs.1;
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::new("insert", size), &kvs, |b, _kv| {
            b.iter(|| {
                let root = bmt.insert(None, &mut keys, values).unwrap();
                criterion::black_box(root);
            });
        });
    }

    group.finish();
    #[cfg(any(feature = "rocksdb"))]
    let path = PathBuf::from("db");
    #[cfg(any(feature = "rocksdb"))]
    remove_dir_all(&path).unwrap();
}

/** Benchmarks 1000, 2000, 5000, 10000 inserts into a tree with existing root */
fn hash_tree_existing_tree_insert_big_benchmark(c: &mut Criterion) {
    let path = PathBuf::from("db");
    let seed = [0xBBu8; 32];
    let mut rng: StdRng = SeedableRng::from_seed(seed);
    let mut group = c.benchmark_group("Big Non-Empty Tree");
    let sizes = vec![1000, 2000, 5000, 10000];
    for size in sizes {
        let (mut keys, values) = prepare_inserts(size, &mut rng);
        let mut bmt = Tree::open(&path, 160).unwrap();
        let root_hash = bmt.insert(None, &mut keys, &values).unwrap();
        let kvs = prepare_inserts(size, &mut rng);
        let mut second_keys = kvs.0.clone();
        let second_values = &kvs.1;
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::new("insert", size), &kvs, |b, _kv| {
            b.iter(|| {
                let root = bmt
                    .insert(Some(&root_hash), &mut second_keys, second_values)
                    .unwrap();
                criterion::black_box(root);
            });
        });
    }
    group.finish();

    #[cfg(any(feature = "rocksdb"))]
    let path = PathBuf::from("db");
    #[cfg(any(feature = "rocksdb"))]
    remove_dir_all(&path).unwrap();
}

/** Benchmarks retrieving 10000 keys from a tree with 10000 keys */
fn get_from_hash_tree_big_benchmark(c: &mut Criterion) {
    let path = PathBuf::from("db");
    let seed = [0xBBu8; 32];
    let mut rng: StdRng = SeedableRng::from_seed(seed);
    c.bench_function("Big Tree Get Benchmark/10000", move |b| {
        let (mut keys, values) = prepare_inserts(10000, &mut rng);

        let mut bmt = Tree::open(&path, 160).unwrap();
        let root_hash = bmt.insert(None, &mut keys, &values).unwrap();

        b.iter(|| {
            let items = bmt.get(&root_hash, &mut keys).unwrap();
            criterion::black_box(items);
        })
    });
    #[cfg(any(feature = "rocksdb"))]
    let path = PathBuf::from("db");
    #[cfg(any(feature = "rocksdb"))]
    remove_dir_all(&path).unwrap();
}

fn remove_from_tree_big_benchmark(c: &mut Criterion) {
    let path = PathBuf::from("db");
    let seed = [0xBBu8; 32];
    let mut rng: StdRng = SeedableRng::from_seed(seed);

    c.bench_function("Big Tree Remove Benchmark/10000", move |b| {
        let (mut keys, values) = prepare_inserts(10000, &mut rng);
        let mut tree = Tree::open(&path.clone(), 160).unwrap();

        let root_hash = tree.insert(None, &mut keys, &values).unwrap();
        b.iter(|| {
            tree.remove(&root_hash).unwrap();
        })
    });
    #[cfg(any(feature = "rocksdb"))]
    let path = PathBuf::from("db");
    #[cfg(any(feature = "rocksdb"))]
    remove_dir_all(&path).unwrap();
}

criterion_group!(
    big_benches,
    hash_tree_empty_tree_insert_big_benchmark,
    hash_tree_existing_tree_insert_big_benchmark,
    get_from_hash_tree_big_benchmark,
    remove_from_tree_big_benchmark
);
criterion_main!(big_benches);

fn prepare_inserts(num_entries: usize, rng: &mut StdRng) -> (Vec<Array<KEY_LEN>>, Vec<Vec<u8>>) {
    let mut keys = Vec::with_capacity(num_entries);
    let mut data = Vec::with_capacity(num_entries);
    for _ in 0..num_entries {
        let mut key_value = [0u8; KEY_LEN];
        rng.fill(&mut key_value);
        keys.push(key_value.into());

        let mut data_value = [0u8; KEY_LEN];
        rng.fill(data_value.as_mut());
        data.push(data_value.to_vec());
    }

    keys.sort();

    (keys, data)
}
