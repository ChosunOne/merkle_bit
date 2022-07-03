use starling::hash_tree::HashTree;
use starling::merkle_bit::BinaryMerkleTreeResult;

fn main() -> BinaryMerkleTreeResult<()> {
    let mut tree: HashTree = HashTree::new(16)?;

    let key = [0x00; 32].into();
    let value = vec![0x00; 32];

    // Inserting and getting from a tree
    let new_root = tree.insert(None, &mut [key], &[value.clone()])?;
    let retrieved_value = tree.get_one(&new_root, &key)?.unwrap();
    assert_eq!(retrieved_value, value);

    // Generating an inclusion proof of an element in the tree
    let inclusion_proof = tree.generate_inclusion_proof(&new_root, key)?;

    // Verifying an inclusion proof.
    HashTree::verify_inclusion_proof(&new_root, key, &value, &inclusion_proof)?;

    // Attempting to get from a removed root will yield None
    tree.remove(&new_root)?;
    let item_map2 = tree.get(&new_root, &mut [key])?;
    assert_eq!(item_map2[&key], None);

    Ok(())
}
