use starling::constants::KEY_LEN;
use starling::merkle_bit::BinaryMerkleTreeResult;

fn main() -> BinaryMerkleTreeResult<()> {
    let mut tree = starling::hash_tree::HashTree::new(16)?;

    let key = [0x00; KEY_LEN];
    let value = vec![0x00; KEY_LEN];

    // Inserting and getting from a tree
    let new_root = tree.insert(None, &mut [key], &mut vec![&value])?;
    let item_map = tree.get(&new_root, &mut [key])?;
    assert_eq!(item_map[&key], Some(value));

    // Attempting to get from a removed root will yield None
    tree.remove(&new_root)?;
    let item_map2 = tree.get(&new_root, &mut [key])?;
    assert_eq!(item_map2[&key], None);
    Ok(())
}
