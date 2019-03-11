#[macro_use]
extern crate criterion;

use std::path::PathBuf;

use criterion::Criterion;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

#[cfg(any(feature = "use_rocksdb"))]
use std::fs::remove_dir_all;

#[cfg(not(any(feature = "use_rocksdb")))]
use starling::hash_tree::HashTree;

#[cfg(feature = "use_rocksdb")]
use starling::rocks_tree::RocksTree;

#[cfg(not(any(feature = "use_rocksdb")))]
type Tree = HashTree<Vec<u8>>;

#[cfg(feature = "use_rocksdb")]
type Tree = RocksTree<Vec<u8>>;

/** Benchmarks 1, 10 , and 100 inserts to a tree with no previous state */
fn hash_tree_empty_tree_insert_benchmark(c: &mut Criterion) {
    let path = PathBuf::from("db");
    let seed = [0xBBu8; 32];
    let mut rng: StdRng = SeedableRng::from_seed(seed);
    c.bench_function_over_inputs("Tree Empty Insert", move |b, index| {
        let prepare = prepare_inserts(1000, &mut rng);
        let key_values = prepare.0;
        let mut keys = key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let data_values = prepare.1;
        let mut data = data_values.iter().collect::<Vec<_>>();
        let mut bmt = Tree::open(&path, 160).unwrap();
        b.iter(|| {
            bmt.insert(None, &mut keys[0..*index].to_vec(), &mut data[0..*index]).unwrap();
        });
    }, vec![1, 10, 100, 200, 500, 1000]);
    #[cfg(any(feature = "use_rocksdb"))]
    let path = PathBuf::from("db");
    #[cfg(any(feature = "use_rocksdb"))]
        remove_dir_all(&path).unwrap();
}

/** Benchmarks 1, 10, and 100 inserts into a tree with existing root */
fn hash_tree_existing_tree_insert_benchmark(c: &mut Criterion) {
    let path = PathBuf::from("db");
    let seed = [0xBBu8; 32];
    let mut rng: StdRng = SeedableRng::from_seed(seed);
    c.bench_function_over_inputs("Tree Non Empty Insert", move |b, index| {
        let prepare = prepare_inserts(4096, &mut rng);
        let key_values = prepare.0;
        let mut keys = key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let data_values = prepare.1;
        let mut data = data_values.iter().collect::<Vec<_>>();

        let mut bmt = Tree::open(&path, 160).unwrap();
        let root_hash = bmt.insert(None, &mut keys, &mut data).unwrap();
        let second = prepare_inserts(1000, &mut rng);
        let mut second_keys = second.0.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let mut second_data = second.1.iter().collect::<Vec<_>>();

        b.iter(|| {
            bmt.insert(Some(&root_hash), &mut second_keys[0..*index], &mut second_data[0..*index]).unwrap();
        })
    }, vec![1, 10, 100, 200, 500, 1000]);
    #[cfg(any(feature = "use_rocksdb"))]
    let path = PathBuf::from("db");
    #[cfg(any(feature = "use_rocksdb"))]
        remove_dir_all(&path).unwrap();
}

/** Benchmarks retrieving 4096 keys from a tree with 4096 keys */
fn get_from_hash_tree_benchmark(c: &mut Criterion) {
    let path = PathBuf::from("db");
    let seed = [0xBBu8; 32];
    let mut rng: StdRng = SeedableRng::from_seed(seed);
    c.bench_function("Tree Get Benchmark", move |b| {
        let prepare = prepare_inserts(4096, &mut rng);
        let key_values = prepare.0;
        let mut keys = key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let data_values = prepare.1;
        let mut data = data_values.iter().collect::<Vec<_>>();
        let mut bmt = Tree::open(&path, 160).unwrap();
        let root_hash = bmt.insert(None, &mut keys, &mut data).unwrap();

        let keys_ = key_values.clone();
        let keys_to_get = keys_.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        b.iter(|| {
            bmt.get(&root_hash, &mut keys_to_get.clone()).unwrap();
        })
    });
    #[cfg(any(feature = "use_rocksdb"))]
        let path = PathBuf::from("db");
    #[cfg(any(feature = "use_rocksdb"))]
    remove_dir_all(&path).unwrap();
}

fn remove_from_tree_benchmark(c: &mut Criterion) {
    let path = PathBuf::from("db");
    let seed = [0xBBu8; 32];
    let mut rng: StdRng = SeedableRng::from_seed(seed);

    c.bench_function("Tree Remove Benchmark", move |b| {
        let prepare = prepare_inserts(4096, &mut rng);
        let mut tree = Tree::open(&path.clone(), 160).unwrap();
        let key_values = prepare.0;
        let mut keys = key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
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


criterion_group!(benches, hash_tree_empty_tree_insert_benchmark, hash_tree_existing_tree_insert_benchmark, get_from_hash_tree_benchmark, remove_from_tree_benchmark);
criterion_main!(benches);

fn prepare_inserts(num_entries: usize, rng: &mut StdRng) -> (Vec<Vec<u8>>, Vec<Vec<u8>>, Vec<Option<Vec<u8>>>) {
    let mut keys = Vec::with_capacity(num_entries);
    let mut data = Vec::with_capacity(num_entries);
    for _ in 0..num_entries {
        let mut key_value = [0u8; 32];
        rng.fill(&mut key_value);
        keys.push(key_value.to_vec());

        let mut data_value = [0u8; 32];
        rng.fill(data_value.as_mut());
        data.push(data_value.to_vec());
    }
    let mut expected_items = vec![];
    for i in 0..num_entries {
        expected_items.push(Some(data[i].clone()));
    }

    keys.sort();

    (keys, data, expected_items)
}

