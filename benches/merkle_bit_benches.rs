#[macro_use]
extern crate criterion;

use criterion::Criterion;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use starling::hash_tree::HashTree;

/** Benchmarks 1, 10 , and 100 inserts to a tree with no previous state */
fn hash_tree_empty_tree_insert_benchmark(c: &mut Criterion){
    c.bench_function_over_inputs("Hash Tree Empty Insert", move |b,index| {
        let seed = [0xBBu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        let prepare = prepare_inserts(1000, &mut rng);
        let key_values = prepare.0;
        let mut keys = vec![];
        let data_values = prepare.1;
        let mut data = vec![];
        for i in 0..data_values.len() {
            data.push(data_values[i].as_ref());
            keys.push(key_values[i].as_ref());
        }
        let mut bmt = HashTree::new(160);
        b.iter(|| bmt.insert(None, &mut keys[0..*index].to_vec(), &mut data[0..*index].to_vec()))
    },vec![1, 10, 100]);
}

/** Benchmarks 1, 10, and 100 inserts into a tree with existing root */
fn hash_tree_existing_tree_insert_benchmark(c: &mut Criterion) {
    c.bench_function_over_inputs("Hash Tree Non Empty Insert", move |b,index| {
        let seed = [0xBBu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        let prepare = prepare_inserts(4096, &mut rng);
        let key_values = prepare.0;
        let mut keys = vec![];
        let data_values = prepare.1;
        let mut data = vec![];
        for i in 0..data_values.len() {
            data.push(data_values[i].as_ref());
            keys.push(key_values[i].as_ref());
        }
        let mut bmt = HashTree::new( 16);
        let root_hash = bmt.insert(None, &mut keys.clone(), &mut data).unwrap();
        let second = prepare_inserts(1000, &mut rng);
        let mut second_data = vec![];
        let mut second_keys =vec![];
        let keys_2 = second.0;
        let data_2 = second.1;
        for i in 0..data_2.len() {
            second_data.push(data_2[i].as_ref());
            second_keys.push(keys_2[i].as_ref());
        }
        b.iter(|| bmt.insert(Some(&root_hash),&mut second_keys[0..*index].to_vec(),&mut second_data[0..*index].to_vec()))
    },vec![1, 10, 100]);
}

/** Benchmarks retrieving 4096 keys from a tree with 4096 keys */
fn get_from_hash_tree_benchmark(c: &mut Criterion) {
    let seed = [0xBBu8; 32];
    let mut rng: StdRng = SeedableRng::from_seed(seed);

    let prepare = prepare_inserts(4096, &mut rng);
    let key_values = prepare.0;
    let mut keys = vec![];
    let data_values = prepare.1;
    let mut data = vec![];
    for i in 0..data_values.len() {
        data.push(data_values[i].as_ref());
        keys.push(key_values[i].as_ref());
    }
    let mut bmt = HashTree::new(16);
    let root_hash = bmt.insert(None, &mut keys.clone(), &mut data).unwrap();
    c.bench_function("Get from BMT Benchmark", move|b|{
        let keys_ = key_values.clone();
        let mut keys_to_get = vec![];
        for i in 0..keys_.len() {
            keys_to_get.push(keys_[i].as_ref());
        }
        b.iter(|| bmt.get(&root_hash,&mut keys_to_get.clone()))
    });
}


criterion_group!(benches, hash_tree_empty_tree_insert_benchmark, hash_tree_existing_tree_insert_benchmark, get_from_hash_tree_benchmark);
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

