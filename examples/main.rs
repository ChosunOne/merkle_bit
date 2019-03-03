extern crate starling;

fn main() {
    let mut tree = starling::tree::HashTree::new(16);

    let key = vec![0x00];
    let value = vec![0x00];

    let new_root = tree.insert(None, &mut [&key], &mut vec![&value]).unwrap();
    let items = tree.get(&new_root, &mut [&key]).unwrap();
    assert_eq!(items, vec![Some(value)]);
}
