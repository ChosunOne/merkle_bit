#[cfg(test)]
#[cfg(all(feature = "default_tree", feature = "use_serialization"))]
pub mod integration_tests {
    use std::path::PathBuf;
    use std::fs::remove_dir_all;

    use starling::merkle_bit::BinaryMerkleTreeResult;
    #[cfg(feature = "use_rocksdb")]
    use starling::rocks_tree::RocksTree;

    #[cfg(not(any(feature = "use_rocksdb")))]
    use starling::hash_tree::HashTree;

    #[cfg(feature = "use_rocksdb")]
    type RealDB = RocksTree<Vec<u8>>;

    #[cfg(not(any(feature = "use_rocksdb")))]
    type RealDB = HashTree;

    #[test]
    fn it_works_with_a_real_database() -> BinaryMerkleTreeResult<()> {
        let retrieved_value;
        let removed_retrieved_value;
        let data = vec![0xFFu8];
        let path = PathBuf::from("db");
        {
            let key = vec![0xAAu8];
            let mut values = vec![data.as_ref()];
            let mut tree = RealDB::open(&path, 160)?;
            let root;
            match tree.insert(None, &mut [&key], &mut values) {
                Ok(r) => root = r,
                Err(e) => {
                    drop(tree);
                    remove_dir_all(&path)?;
                    panic!("{:?}", e.description());
                }
            }
            match tree.get(&root, &mut [&key]) {
                Ok(v) => retrieved_value = v,
                Err(e) => {
                    drop(tree);
                    remove_dir_all(&path)?;
                    panic!("{:?}", e.description());
                }
            }
            match tree.remove(&root) {
                Ok(_) => {},
                Err(e) => {
                    drop(tree);
                    remove_dir_all(&path)?;
                    panic!("{:?}", e.description());
                }
            }
            match tree.get(&root, &mut [&key]) {
                Ok(v) => removed_retrieved_value = v,
                Err(e) => {
                    drop(tree);
                    remove_dir_all(&path)?;
                    panic!("{:?}", e.description());
                }
            }
        }
        remove_dir_all(&path)?;
        assert_eq!(retrieved_value, vec![Some(data)]);
        assert_eq!(removed_retrieved_value, vec![None]);
        Ok(())
    }
}
