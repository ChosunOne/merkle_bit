#[macro_use]
extern crate criterion;

#[cfg(any(feature = "use_rocksdb"))]
use std::fs::remove_dir_all;
use std::path::PathBuf;

use criterion::Criterion;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use starling::constants::KEY_LEN;
#[cfg(not(any(feature = "use_rocksdb")))]
use starling::hash_tree::HashTree;
#[cfg(feature = "use_rocksdb")]
use starling::rocks_tree::RocksTree;

#[cfg(not(any(feature = "use_rocksdb")))]
type Tree = HashTree<[u8; KEY_LEN], Vec<u8>>;

#[cfg(feature = "use_rocksdb")]
type Tree = RocksTree<[u8; KEY_LEN], Vec<u8>>;

/** Benchmarks 1000, 2000, 5000, 10000 inserts to a tree with no previous state */
fn hash_tree_empty_tree_insert_big_benchmark(c: &mut Criterion) {
    let path = PathBuf::from("db");
    let seed = [0xBBu8; KEY_LEN];
    let mut rng: StdRng = SeedableRng::from_seed(seed);
    c.bench_function_over_inputs(
        "Big Tree Empty Insert",
        move |b, index| {
            let (mut keys, values) = prepare_inserts(10000, &mut rng);

            let mut bmt = Tree::open(&path, 160).unwrap();
            b.iter(|| {
                let root = bmt
                    .insert(None, &mut keys[0..*index], &values[0..*index])
                    .unwrap();
                criterion::black_box(root);
            });
        },
        vec![1000, 2000, 5000, 10000],
    );
    #[cfg(any(feature = "use_rocksdb"))]
    let path = PathBuf::from("db");
    #[cfg(any(feature = "use_rocksdb"))]
    remove_dir_all(&path).unwrap();
}

/** Benchmarks 1000, 2000, 5000, 10000 inserts into a tree with existing root */
fn hash_tree_existing_tree_insert_big_benchmark(c: &mut Criterion) {
    let path = PathBuf::from("db");
    let seed = [0xBBu8; KEY_LEN];
    let mut rng: StdRng = SeedableRng::from_seed(seed);
    c.bench_function_over_inputs(
        "Big Tree Non Empty Insert",
        move |b, index| {
            let (mut keys, values) = prepare_inserts(10000, &mut rng);

            let mut bmt = Tree::open(&path, 160).unwrap();
            let root_hash = bmt.insert(None, &mut keys, &values).unwrap();
            let (mut second_keys, second_values) = prepare_inserts(10000, &mut rng);

            b.iter(|| {
                let root = bmt
                    .insert(
                        Some(&root_hash),
                        &mut second_keys[0..*index],
                        &second_values[0..*index],
                    )
                    .unwrap();
                criterion::black_box(root);
            })
        },
        vec![1000, 2000, 5000, 10000],
    );
    #[cfg(any(feature = "use_rocksdb"))]
    let path = PathBuf::from("db");
    #[cfg(any(feature = "use_rocksdb"))]
    remove_dir_all(&path).unwrap();
}

/** Benchmarks retrieving 10000 keys from a tree with 10000 keys */
fn get_from_hash_tree_big_benchmark(c: &mut Criterion) {
    let path = PathBuf::from("db");
    let seed = [0xBBu8; KEY_LEN];
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
    #[cfg(any(feature = "use_rocksdb"))]
    let path = PathBuf::from("db");
    #[cfg(any(feature = "use_rocksdb"))]
    remove_dir_all(&path).unwrap();
}

fn remove_from_tree_big_benchmark(c: &mut Criterion) {
    let path = PathBuf::from("db");
    let seed = [0xBBu8; KEY_LEN];
    let mut rng: StdRng = SeedableRng::from_seed(seed);

    c.bench_function("Big Tree Remove Benchmark/10000", move |b| {
        let (mut keys, values) = prepare_inserts(10000, &mut rng);
        let mut tree = Tree::open(&path.clone(), 160).unwrap();

        let root_hash = tree.insert(None, &mut keys, &values).unwrap();
        b.iter(|| {
            tree.remove(&root_hash).unwrap();
        })
    });
    #[cfg(any(feature = "use_rocksdb"))]
    let path = PathBuf::from("db");
    #[cfg(any(feature = "use_rocksdb"))]
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

fn prepare_inserts(num_entries: usize, rng: &mut StdRng) -> (Vec<[u8; KEY_LEN]>, Vec<Vec<u8>>) {
    let mut keys = Vec::with_capacity(num_entries);
    let mut data = Vec::with_capacity(num_entries);
    for _ in 0..num_entries {
        let mut key_value = [0u8; KEY_LEN];
        rng.fill(&mut key_value);
        keys.push(key_value);

        let mut data_value = [0u8; KEY_LEN];
        rng.fill(data_value.as_mut());
        data.push(data_value.to_vec());
    }

    keys.sort();

    (keys, data)
}
