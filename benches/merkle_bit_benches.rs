#[macro_use]
extern crate criterion;

use criterion::Criterion;
use starling::tree::HashTree;
// Hash Tree Benchmarks
fn empty_tree_insert_benchmark(c: &mut Criterion){
    let key: Vec<u8> = vec![0x00u8, 0x81u8, 0xA3u8];
    let value: Vec<u8> = vec![0xDDu8];
    c.bench_function("Hash Tree Empty Insert", move |b| {
        let mut tree = HashTree::new(8);
        b.iter(|| tree.insert(None, vec![key.as_ref()], &[value.as_ref()]).unwrap())
    });
}
// fn existing_tree_insert_benchmark(c: &mut Criterion) {}
// fn get_from_tree_benchmark(c: &mut Criterion) {}
// fn get_from_tree_worst_case_benchmark(c: &mut Criterion) {}

criterion_group!(benches, empty_tree_insert_benchmark);
criterion_main!(benches);