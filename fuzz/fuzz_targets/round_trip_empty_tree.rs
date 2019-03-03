#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate starling;

fuzz_target!(|data: &[u8]| {
    // fuzzed code goes here
    let key_and_value = get_key_and_value(data);
    let mut key = key_and_value.0.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
    let mut val = key_and_value.1.iter().collect::<Vec<_>>();
    let mut bmt = starling::tree::HashTree::new(16);
    let root = bmt.insert(None, &mut key, &mut val).unwrap();
    let items = bmt.get(&root, &mut key).unwrap();
    assert_eq!(items, vec![Some(key_and_value.1[0].clone())]);
});

fn get_key_and_value(data: &[u8]) -> (Vec<Vec<u8>>, Vec<Vec<u8>>) {
    if data.is_empty() || data.len() < 2 {
        return (vec![vec![0]], vec![vec![0]])
    }
    let split = data.split_at(data.len() / 2);
    (vec![split.0.to_vec()], vec![split.1.to_vec()])
}