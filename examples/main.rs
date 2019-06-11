use starling::constants::KEY_LEN;
use starling::merkle_bit::BinaryMerkleTreeResult;
use starling::hash_tree::HashTree;

fn main() -> BinaryMerkleTreeResult<()> {
    let mut tree: HashTree<[u8; KEY_LEN], Vec<u8>> = HashTree::new(16)?;

    let key = [0x00; KEY_LEN];
    let value = vec![0x00; KEY_LEN];

    // Inserting and getting from a tree
    let new_root = tree.insert(None, &mut [key], &vec![value.clone()])?;
    let retrieved_value = tree.get_one(&new_root, &key)?.unwrap();
    assert_eq!(retrieved_value, value.clone());

    // Generating an inclusion proof of an element in the tree
    let inclusion_proof = tree.generate_inclusion_proof(&new_root, key)?;

    // Verifying an inclusion proof.  Note that the verifying tree can be empty, though it must
    // be the same type as the tree that generated the proof.
    let verifier: HashTree<[u8; KEY_LEN], Vec<u8>> = HashTree::new(16)?;
    verifier.verify_inclusion_proof(&new_root, key, &value, &inclusion_proof)?;

    // Attempting to get from a removed root will yield None
    tree.remove(&new_root)?;
    let item_map2 = tree.get(&new_root, &mut [key])?;
    assert_eq!(item_map2[&key], None);

    Ok(())
}
