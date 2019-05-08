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
type Tree = HashTree<Vec<u8>>;

#[cfg(feature = "use_rocksdb")]
type Tree = RocksTree<Vec<u8>>;

/** Benchmarks 1000, 2000, 5000, 10000 inserts to a tree with no previous state */
fn hash_tree_empty_tree_insert_big_benchmark(c: &mut Criterion) {
    let path = PathBuf::from("db");
    let seed = [0xBBu8; KEY_LEN];
    let mut rng: StdRng = SeedableRng::from_seed(seed);
    c.bench_function_over_inputs(
        "Big Tree Empty Insert",
        move |b, index| {
            let prepare = prepare_inserts(10000, &mut rng);
            let key_values = prepare.0;
            let mut keys = key_values.iter().collect::<Vec<_>>();
            let data_values = prepare.1;
            let mut data = data_values.iter().collect::<Vec<_>>();
            let mut bmt = Tree::open(&path, 160).unwrap();
            b.iter(|| {
                let root = bmt
                    .insert(None, &mut keys[0..*index], &mut data[0..*index])
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
            let prepare = prepare_inserts(10000, &mut rng);
            let key_values = prepare.0;
            let mut keys = key_values.iter().collect::<Vec<_>>();
            let data_values = prepare.1;
            let mut data = data_values.iter().collect::<Vec<_>>();

            let mut bmt = Tree::open(&path, 160).unwrap();
            let root_hash = bmt.insert(None, &mut keys, &mut data).unwrap();
            let second = prepare_inserts(10000, &mut rng);
            let mut second_keys = second.0.iter().collect::<Vec<_>>();
            let mut second_data = second.1.iter().collect::<Vec<_>>();

            b.iter(|| {
                let root = bmt
                    .insert(
                        Some(&root_hash),
                        &mut second_keys[0..*index],
                        &mut second_data[0..*index],
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
        let prepare = prepare_inserts(10000, &mut rng);
        let key_values = prepare.0;
        let mut keys = key_values.iter().collect::<Vec<_>>();
        let data_values = prepare.1;
        let mut data = data_values.iter().collect::<Vec<_>>();
        let mut bmt = Tree::open(&path, 160).unwrap();
        let root_hash = bmt.insert(None, &mut keys, &mut data).unwrap();

        let keys_ = key_values.clone();
        let mut keys_to_get = keys_.iter().collect::<Vec<_>>();
        b.iter(|| {
            let items = bmt.get(&root_hash, &mut keys_to_get).unwrap();
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
        let prepare = prepare_inserts(10000, &mut rng);
        let mut tree = Tree::open(&path.clone(), 160).unwrap();
        let key_values = prepare.0;
        let mut keys = key_values.iter().collect::<Vec<_>>();
        let data_values = prepare.1;
        let mut data = data_values.iter().collect::<Vec<_>>();
        let root_hash = tree.insert(None, &mut keys, &mut data).unwrap();
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
