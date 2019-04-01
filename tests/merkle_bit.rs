#[cfg(test)]
pub mod integration_tests {
    #[cfg(any(feature = "use_serialization"))]
    use std::error::Error;
    use std::path::PathBuf;

    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    use starling::merkle_bit::BinaryMerkleTreeResult;
    #[cfg(feature = "use_rocksdb")]
    use starling::rocks_tree::RocksTree;

    #[cfg(not(any(feature = "use_rocksdb")))]
    use starling::hash_tree::HashTree;

    #[cfg(feature = "use_rocksdb")]
    type Tree = RocksTree<Vec<u8>>;

    #[cfg(not(any(feature = "use_rocksdb")))]
    type Tree = HashTree<Vec<u8>>;

    #[test]
    #[cfg(feature = "use_serialization")]
    fn it_works_with_a_real_database() -> BinaryMerkleTreeResult<()> {
        let seed = [0x00u8; 32];
        let path = generate_path(seed);
        let key = [0xAAu8; 32];
        let retrieved_value;
        let removed_retrieved_value;
        let data = vec![0xFFu8];
        {
            let mut values = vec![&data];
            let mut tree = Tree::open(&path, 160)?;
            let root;
            match tree.insert(None, &mut [&key], &mut values) {
                Ok(r) => root = r,
                Err(e) => {
                    drop(tree);
                    tear_down(&path);
                    panic!("{:?}", e.description());
                }
            }
            match tree.get(&root, &mut [&key]) {
                Ok(v) => retrieved_value = v,
                Err(e) => {
                    drop(tree);
                    tear_down(&path);
                    panic!("{:?}", e.description());
                }
            }
            match tree.remove(&root) {
                Ok(_) => {}
                Err(e) => {
                    drop(tree);
                    tear_down(&path);
                    panic!("{:?}", e.description());
                }
            }
            match tree.get(&root, &mut [&key]) {
                Ok(v) => removed_retrieved_value = v,
                Err(e) => {
                    drop(tree);
                    tear_down(&path);
                    panic!("{:?}", e.description());
                }
            }
        }
        tear_down(&path);
        assert_eq!(retrieved_value[&key[..]], Some(data));
        assert_eq!(removed_retrieved_value[&key[..]], None);
        Ok(())
    }

    #[test]
    fn it_gets_an_item_out_of_a_simple_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x01u8; 32];
        let path = generate_path(seed);
        let key = [0xAAu8; 32];
        let value = vec![0xFFu8];

        let mut bmt = Tree::open(&path, 160)?;
        let root = bmt.insert(None, &mut [&key[..]], &mut vec![&value])?;
        let result = bmt.get(&root, &mut vec![&key[..]])?;
        assert_eq!(result[&key[..]], Some(vec![0xFFu8]));
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_fails_to_get_from_empty_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x02u8; 32];
        let path = generate_path(seed);
        let key = [0x00u8; 32];
        let root_key = [0x01u8; 32];

        let bmt = Tree::open(&path, 160)?;
        let items = bmt.get(&root_key, &mut [&key[..]])?;
        let expected_item = None;
        assert_eq!(items[&key[..]], expected_item);
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_fails_to_get_a_nonexistent_item() -> BinaryMerkleTreeResult<()> {
        let seed = [0x03u8; 32];
        let path = generate_path(seed);
        let key = [0xAAu8; 32];
        let value = vec![0xFFu8];

        let mut bmt = Tree::open(&path, 160)?;
        let root = bmt.insert(None, &mut [&key[..]], &mut [&value])?;

        let nonexistent_key = [0xAB; 32];
        let items = bmt.get(&root, &mut [&nonexistent_key[..]])?;
        assert_eq!(items[&nonexistent_key[..]], None);
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_gets_items_from_a_small_balanced_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x04u8; 32];
        let path = generate_path(seed);
        let mut keys = Vec::with_capacity(8);
        let mut values = Vec::with_capacity(8);
        for i in 0..8 {
            keys.push([i << 5u8; 32]);
            values.push(vec![i; 32]);
        }
        let mut get_keys = keys.iter().map(|x| &x[..]).collect::<Vec<_>>();

        let mut bmt = Tree::open(&path, 3)?;
        let mut insert_values = values.iter().collect::<Vec<_>>();
        let root_hash = bmt.insert(None, &mut get_keys, &mut insert_values)?;

        let items = bmt.get(&root_hash, &mut get_keys)?;
        for (key, value) in get_keys.into_iter().zip(values.iter()) {
            assert_eq!(Some(value.clone()), items[&key[..]])
        }
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_gets_items_from_a_small_unbalanced_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x05u8; 32];
        let path = generate_path(seed);
        let mut keys = Vec::with_capacity(7);
        let mut values: Vec<Vec<u8>> = Vec::with_capacity(7);
        for i in 0..7 {
            keys.push([i << 5u8; 32]);
            values.push(vec![i; 32]);
        }
        let mut get_keys = keys.iter().map(|x| &x[..]).collect::<Vec<_>>();
        let mut insert_values = values.iter().collect::<Vec<_>>();
        let mut bmt = Tree::open(&path, 3)?;

        let root_hash = bmt.insert(None, &mut get_keys, &mut insert_values)?;
        let items = bmt.get(&root_hash, &mut get_keys)?;
        for (key, value) in keys.iter().zip(values.iter()) {
            assert_eq!(items[&key[..]], Some(value.clone()))
        }
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_gets_items_from_a_medium_balanced_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x06u8; 32];
        let path = generate_path(seed);

        let num_leaves = 256;
        let mut keys = Vec::with_capacity(num_leaves);
        let mut values: Vec<Vec<u8>> = Vec::with_capacity(num_leaves);
        for i in 0..num_leaves {
            keys.push([i as u8; 32]);
            values.push(vec![i as u8; 32]);
        }

        let mut get_keys = keys.iter().map(|x| &x[..]).collect::<Vec<_>>();
        let mut insert_values = values.iter().collect::<Vec<_>>();

        let mut bmt = Tree::open(&path, 8)?;
        let root_hash = bmt.insert(None, &mut get_keys, &mut insert_values)?;

        let items = bmt.get(&root_hash, &mut get_keys)?;
        for (key, value) in keys.iter().zip(values.iter()) {
            assert_eq!(items[&key[..]], Some(value.clone()))
        }
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_gets_items_from_a_medium_unbalanced_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x07u8; 32];
        let path = generate_path(seed);
        let num_leaves = 255;
        let mut keys = Vec::with_capacity(num_leaves);
        let mut values: Vec<Vec<u8>> = Vec::with_capacity(num_leaves);
        for i in 0..num_leaves {
            keys.push([i as u8; 32]);
            values.push(vec![i as u8; 32]);
        }

        let mut get_keys = keys.iter().map(|x| &x[..]).collect::<Vec<_>>();
        let mut insert_values = values.iter().collect::<Vec<_>>();

        let mut bmt = Tree::open(&path, 8)?;
        let root_hash = bmt.insert(None, &mut get_keys, &mut insert_values)?;

        let items = bmt.get(&root_hash, &mut get_keys)?;
        for (key, value) in keys.iter().zip(values.iter()) {
            assert_eq!(items[&key[..]], Some(value.clone()))
        }
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_gets_items_from_a_large_balanced_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x08u8; 32];
        let path = generate_path(seed);

        #[cfg(not(any(feature = "use_groestl")))]
        let num_leaves = 8196;
        #[cfg(feature = "use_groestl")]
        let num_leaves = 1024;

        let mut keys = Vec::with_capacity(num_leaves);
        let mut values: Vec<Vec<u8>> = Vec::with_capacity(num_leaves);
        for i in 0..num_leaves {
            let mut key = [0u8; 32];
            key[0] = (i >> 8) as u8;
            key[1] = (i & 0xFF) as u8;
            values.push(key.to_vec());
            keys.push(key);
        }

        let mut get_keys = keys.iter().map(|x| &x[..]).collect::<Vec<_>>();
        let mut insert_values = values.iter().collect::<Vec<_>>();

        let mut bmt = Tree::open(&path, 16)?;
        let root_hash = bmt.insert(None, &mut get_keys, &mut insert_values)?;

        let items = bmt.get(&root_hash, &mut get_keys)?;
        for (key, value) in keys.iter().zip(values.iter()) {
            assert_eq!(items[&key[..]], Some(value.clone()))
        }
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_gets_items_from_a_large_unbalanced_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x09u8; 32];
        let path = generate_path(seed);

        #[cfg(not(any(feature = "use_groestl")))]
        let num_leaves = 8195;
        #[cfg(feature = "use_groestl")]
        let num_leaves = 1023;
        let mut keys = Vec::with_capacity(num_leaves);
        let mut values: Vec<Vec<u8>> = Vec::with_capacity(num_leaves);
        for i in 0..num_leaves {
            let mut key = [0u8; 32];
            key[0] = (i >> 8) as u8;
            key[1] = (i & 0xFF) as u8;
            values.push(key.to_vec());
            keys.push(key);
        }

        let mut get_keys = keys.iter().map(|x| &x[..]).collect::<Vec<_>>();
        let mut insert_values = values.iter().collect::<Vec<_>>();

        let mut bmt = Tree::open(&path, 16)?;
        let root_hash = bmt.insert(None, &mut get_keys, &mut insert_values)?;

        let items = bmt.get(&root_hash, &mut get_keys)?;
        for (key, value) in keys.iter().zip(values.iter()) {
            assert_eq!(items[&key[..]], Some(value.clone()))
        }
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_gets_items_from_a_complex_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x10u8; 32];
        let path = generate_path(seed);

        // Tree description
        // Node (Letter)
        // Key (Number)
        // Value (Number)
        //
        // A     B      C      D     E     F     G     H     I     J     K     L     M     N     O     P
        // 0x00  0x40, 0x41, 0x60, 0x68, 0x70, 0x71, 0x72, 0x80, 0xC0, 0xC1, 0xE0, 0xE1, 0xE2, 0xF0, 0xF8
        // None, None, None, 0x01, 0x02, None, None, None, 0x03, None, None, None, None, None, 0x04, None
        let pop_key_d = [0x60u8; 32]; // 0110_0000   96 (Dec)
        let pop_key_e = [0x68u8; 32]; // 0110_1000  104 (Dec)
        let pop_key_i = [0x80u8; 32]; // 1000_0000  128 (Dec)
        let pop_key_o = [0xF0u8; 32]; // 1111_0000  240 (Dec)

        let mut populated_keys = [
            &pop_key_d[..],
            &pop_key_e[..],
            &pop_key_i[..],
            &pop_key_o[..],
        ];

        let pop_value_d = vec![0x01u8];
        let pop_value_e = vec![0x02u8];
        let pop_value_i = vec![0x03u8];
        let pop_value_o = vec![0x04u8];

        let mut populated_values = vec![&pop_value_d, &pop_value_e, &pop_value_i, &pop_value_o];

        let mut bmt = Tree::open(&path, 5)?;
        let root_node = bmt.insert(None, &mut populated_keys, &mut populated_values)?;

        let key_a = [0x00u8; 32]; // 0000_0000     0 (Dec)
        let key_b = [0x40u8; 32]; // 0100_0000    64 (Dec)
        let key_c = [0x41u8; 32]; // 0100_0001    65 (Dec)
        let key_f = [0x70u8; 32]; // 0111_0000   112 (Dec)
        let key_g = [0x71u8; 32]; // 0111_0001   113 (Dec)
        let key_h = [0x72u8; 32]; // 0111_0010   114 (Dec)
        let key_j = [0xC0u8; 32]; // 1100_0000   192 (Dec)
        let key_k = [0xC1u8; 32]; // 1100_0001   193 (Dec)
        let key_l = [0xE0u8; 32]; // 1110_0000   224 (Dec)
        let key_m = [0xE1u8; 32]; // 1110_0001   225 (Dec)
        let key_n = [0xE2u8; 32]; // 1110_0010   226 (Dec)
        let key_p = [0xF8u8; 32]; // 1111_1000   248 (Dec)

        let mut keys = vec![
            &key_a[..],
            &key_b[..],
            &key_c[..],
            &pop_key_d[..],
            &pop_key_e[..],
            &key_f[..],
            &key_g[..],
            &key_h[..],
            &pop_key_i[..],
            &key_j[..],
            &key_k[..],
            &key_l[..],
            &key_m[..],
            &key_n[..],
            &pop_key_o[..],
            &key_p[..],
        ];

        let expected_values = vec![
            None,
            None,
            None,
            Some(pop_value_d),
            Some(pop_value_e),
            None,
            None,
            None,
            Some(pop_value_i),
            None,
            None,
            None,
            None,
            None,
            Some(pop_value_o),
            None,
        ];

        let items = bmt.get(&root_node, &mut keys)?;
        for (key, value) in keys.iter().zip(expected_values.into_iter()) {
            assert_eq!(items[&key[..]], value);
        }
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_returns_the_same_number_of_values_as_keys() -> BinaryMerkleTreeResult<()> {
        let seed = [0x11u8; 32];
        let path = generate_path(seed);

        let initial_key = [0x00u8; 32];
        let initial_value = vec![0xFFu8];

        let mut keys = Vec::with_capacity(256);
        for i in 0..256 {
            keys.push([i as u8]);
        }

        let mut get_keys = keys.iter().map(|x| &x[..]).collect::<Vec<_>>();

        let mut bmt = Tree::open(&path, 3)?;
        let root_node = bmt.insert(None, &mut [&initial_key], &mut vec![&initial_value])?;

        let items = bmt.get(&root_node, &mut get_keys)?;
        for key in get_keys.iter() {
            if **key == initial_key[..] {
                assert_eq!(items[&key[..]], Some(initial_value.clone()));
            } else {
                assert_eq!(items[&key[..]], None);
            }
        }
        assert_eq!(items.len(), 256);
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_inserts_two_leaf_nodes_into_empty_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x12u8; 32];
        let path = generate_path(seed);

        let key_values = vec![[0x00u8; 32], [0x01u8; 32]];
        let mut keys = key_values.iter().map(|x| &x[..]).collect::<Vec<_>>();
        let data_values = vec![vec![0x02u8], vec![0x03u8]];
        let mut data = data_values.iter().collect::<Vec<_>>();

        let mut bmt = Tree::open(&path, 3)?;
        let root_hash = bmt.insert(None, &mut keys, &mut data)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        for (key, value) in keys.iter().zip(data_values.iter()) {
            assert_eq!(items[&key[..]], Some(value.clone()))
        }
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_inserts_two_leaf_nodes_into_empty_tree_with_first_bit_split() -> BinaryMerkleTreeResult<()>
    {
        let seed = [0x13u8; 32];
        let path = generate_path(seed);

        let key_values = vec![[0x00u8; 32], [0x80u8; 32]];
        let mut keys = key_values.iter().map(|x| &x[..]).collect::<Vec<_>>();
        let data_values = vec![vec![0x02u8], vec![0x03u8]];
        let mut data = data_values.iter().collect::<Vec<_>>();

        let mut bmt = Tree::open(&path, 3)?;
        let root_hash = bmt.insert(None, &mut keys, &mut data)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        for (key, value) in keys.iter().zip(data_values.iter()) {
            assert_eq!(items[&key[..]], Some(value.clone()))
        }
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_inserts_a_leaf_node_into_empty_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x14u8; 32];
        let path = generate_path(seed);

        let key = [0xAAu8; 32];
        let data = vec![0xBBu8];

        let mut bmt = Tree::open(&path, 3)?;
        let new_root_hash = bmt.insert(None, &mut [&key[..]], &mut vec![&data])?;
        let items = bmt.get(&new_root_hash, &mut vec![&key[..]])?;
        assert_eq!(items[&key[..]], Some(data));
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_inserts_multiple_leaf_nodes_into_empty_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x15u8; 32];
        let path = generate_path(seed);

        let key_values = vec![
            [0xAAu8; 32], // 1010_1010
            [0xBBu8; 32], // 1011_1011
            [0xCCu8; 32],
        ]; // 1100_1100
        let mut keys = key_values.iter().map(|x| &x[..]).collect::<Vec<_>>();
        let data_values = vec![vec![0xDDu8], vec![0xEEu8], vec![0xFFu8]];
        let mut data = data_values.iter().collect::<Vec<_>>();

        let mut bmt = Tree::open(&path, 3)?;
        let root_hash = bmt.insert(None, &mut keys, &mut data)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        for (key, value) in keys.iter().zip(data_values.iter()) {
            assert_eq!(items[&key[..]], Some(value.clone()))
        }
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_inserts_a_small_even_amount_of_nodes_into_empty_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x016u8; 32];
        let path = generate_path(seed);

        let seed = [0xAAu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        let prepare = prepare_inserts(32, &mut rng);

        let key_values = prepare.0;
        let mut keys = key_values.iter().map(|x| &x[..]).collect::<Vec<_>>();
        let data_values = prepare.1;
        let mut data = data_values.iter().collect::<Vec<_>>();

        let mut bmt = Tree::open(&path, 16)?;
        let root_hash = bmt.insert(None, &mut keys, &mut data)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        for (key, value) in keys.iter().zip(data.into_iter()) {
            assert_eq!(items[&key[..]], Some(value.clone()))
        }
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_inserts_a_small_odd_amount_of_nodes_into_empty_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x17u8; 32];
        let path = generate_path(seed);

        let seed = [0xBBu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        let prepare = prepare_inserts(31, &mut rng);

        let key_values = prepare.0;
        let mut keys = key_values.iter().map(|x| &x[..]).collect::<Vec<_>>();
        let data_values = prepare.1;
        let mut data = data_values.iter().collect::<Vec<_>>();

        let mut bmt = Tree::open(&path, 16)?;
        let root_hash = bmt.insert(None, &mut keys, &mut data)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        for (key, value) in keys.iter().zip(data_values.iter()) {
            assert_eq!(items[&key[..]], Some(value.clone()))
        }
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_inserts_a_medium_even_amount_of_nodes_into_empty_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x18u8; 32];
        let path = generate_path(seed);

        let seed = [0xBBu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        let prepare = prepare_inserts(256, &mut rng);

        let key_values = prepare.0;
        let mut keys = key_values.iter().map(|x| &x[..]).collect::<Vec<_>>();
        let data_values = prepare.1;
        let mut data = data_values.iter().collect::<Vec<_>>();

        let mut bmt = Tree::open(&path, 16)?;
        let root_hash = bmt.insert(None, &mut keys, &mut data)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        for (key, value) in keys.iter().zip(data_values.iter()) {
            assert_eq!(items[&key[..]], Some(value.clone()))
        }
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_inserts_a_medium_odd_amount_of_nodes_into_empty_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x19u8; 32];
        let path = generate_path(seed);

        let seed = [0xBBu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        let prepare = prepare_inserts(255, &mut rng);

        let key_values = prepare.0;
        let mut keys = key_values.iter().map(|x| &x[..]).collect::<Vec<_>>();
        let data_values = prepare.1;
        let mut data = data_values.iter().collect::<Vec<_>>();

        let mut bmt = Tree::open(&path, 16)?;
        let root_hash = bmt.insert(None, &mut keys, &mut data)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        for (key, value) in keys.iter().zip(data_values.iter()) {
            assert_eq!(items[&key[..]], Some(value.clone()))
        }
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_inserts_a_large_even_amount_of_nodes_into_empty_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x20u8; 32];
        let path = generate_path(seed);

        let seed = [0xBBu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        #[cfg(not(any(feature = "use_groestl")))]
        let prepare = prepare_inserts(4096, &mut rng);
        #[cfg(feature = "use_groestl")]
        let prepare = prepare_inserts(256, &mut rng);

        let key_values = prepare.0;
        let mut keys = key_values.iter().map(|x| &x[..]).collect::<Vec<_>>();
        let data_values = prepare.1;
        let mut data = data_values.iter().collect::<Vec<_>>();

        let mut bmt = Tree::open(&path, 16)?;
        let root_hash = bmt.insert(None, &mut keys, &mut data)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        for (key, value) in keys.iter().zip(data_values.iter()) {
            assert_eq!(items[&key[..]], Some(value.clone()))
        }
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_inserts_a_large_odd_amount_of_nodes_into_empty_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x21u8; 32];
        let path = generate_path(seed);

        let seed = [0xBBu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        #[cfg(not(any(feature = "use_groestl")))]
        let prepare = prepare_inserts(4095, &mut rng);
        #[cfg(feature = "use_groestl")]
        let prepare = prepare_inserts(256, &mut rng);

        let key_values = prepare.0;
        let mut keys = key_values.iter().map(|x| &x[..]).collect::<Vec<_>>();
        let data_values = prepare.1;
        let mut data = data_values.iter().collect::<Vec<_>>();

        let mut bmt = Tree::open(&path, 16)?;
        let root_hash = bmt.insert(None, &mut keys, &mut data)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        for (key, value) in keys.iter().zip(data_values.iter()) {
            assert_eq!(items[&key[..]], Some(value.clone()))
        }
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_inserts_a_leaf_node_into_a_tree_with_one_item() -> BinaryMerkleTreeResult<()> {
        let seed = [0x22u8; 32];
        let path = generate_path(seed);

        let first_key = vec![0xAAu8; 32];
        let first_data = vec![0xBBu8];

        let second_key = vec![0xCCu8; 32];
        let second_data = vec![0xDDu8];

        let mut bmt = Tree::open(&path, 3)?;
        let new_root_hash = bmt.insert(
            None,
            &mut vec![first_key.as_ref()],
            &mut vec![first_data.as_ref()],
        )?;
        let second_root_hash = bmt.insert(
            Some(&new_root_hash),
            &mut vec![second_key.as_ref()],
            &mut vec![second_data.as_ref()],
        )?;

        let items = bmt.get(
            &second_root_hash,
            &mut vec![first_key.as_ref(), second_key.as_ref()],
        )?;
        assert_eq!(items[&first_key[..]], Some(first_data));
        assert_eq!(items[&second_key[..]], Some(second_data));
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_inserts_multiple_leaf_nodes_into_a_small_tree_with_existing_items(
    ) -> BinaryMerkleTreeResult<()> {
        let seed = [0x23u8; 32];
        let path = generate_path(seed);

        let seed = [
            0x4d, 0x1b, 0xf8, 0xad, 0x2d, 0x5d, 0x2e, 0xcb, 0x59, 0x75, 0xc4, 0xb9, 0x4d, 0xf9,
            0xab, 0x5e, 0xf5, 0x12, 0xd4, 0x5c, 0x3d, 0xa0, 0x73, 0x4b, 0x65, 0x5e, 0xc3, 0x82,
            0xcb, 0x6c, 0xc0, 0x66,
        ];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        let num_inserts = 2;
        let prepare_initial = prepare_inserts(num_inserts, &mut rng);
        let initial_key_values = prepare_initial.0;
        let mut initial_keys = initial_key_values
            .iter()
            .map(|x| &x[..])
            .collect::<Vec<_>>();
        let initial_data_values = prepare_initial.1;
        let mut initial_data = initial_data_values.iter().collect::<Vec<_>>();

        let mut bmt = Tree::open(&path, 160)?;
        let first_root_hash = bmt.insert(None, &mut initial_keys, &mut initial_data)?;

        let prepare_added = prepare_inserts(num_inserts, &mut rng);
        let added_key_values = prepare_added.0;
        let mut added_keys = added_key_values.iter().map(|x| &x[..]).collect::<Vec<_>>();
        let added_data_values = prepare_added.1;
        let mut added_data = added_data_values.iter().collect::<Vec<_>>();

        let second_root_hash =
            bmt.insert(Some(&first_root_hash), &mut added_keys, &mut added_data)?;

        let first_items = bmt.get(&first_root_hash, &mut initial_keys)?;
        let second_items = bmt.get(&second_root_hash, &mut added_keys)?;

        for (key, value) in initial_keys.iter().zip(initial_data_values.iter()) {
            assert_eq!(first_items[&key[..]], Some(value.clone()));
        }
        for (key, value) in added_keys.iter().zip(added_data_values.iter()) {
            assert_eq!(second_items[&key[..]], Some(value.clone()));
        }
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_inserts_multiple_leaf_nodes_into_a_tree_with_existing_items() -> BinaryMerkleTreeResult<()>
    {
        let seed = [0x24u8; 32];
        let path = generate_path(seed);

        let seed = [0xCAu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        #[cfg(not(any(feature = "use_groestl")))]
        let num_inserts = 4096;
        #[cfg(feature = "use_groestl")]
        let num_inserts = 256;
        let prepare_initial = prepare_inserts(num_inserts, &mut rng);
        let initial_key_values = prepare_initial.0;
        let mut initial_keys = initial_key_values
            .iter()
            .map(|x| &x[..])
            .collect::<Vec<_>>();
        let initial_data_values = prepare_initial.1;
        let mut initial_data = initial_data_values.iter().collect::<Vec<_>>();

        let mut bmt = Tree::open(&path, 160)?;
        let first_root_hash = bmt.insert(None, &mut initial_keys, &mut initial_data)?;

        let prepare_added = prepare_inserts(num_inserts, &mut rng);
        let added_key_values = prepare_added.0;
        let mut added_keys = added_key_values.iter().map(|x| &x[..]).collect::<Vec<_>>();
        let added_data_values = prepare_added.1;
        let mut added_data = added_data_values.iter().collect::<Vec<_>>();

        let second_root_hash =
            bmt.insert(Some(&first_root_hash), &mut added_keys, &mut added_data)?;

        let first_items = bmt.get(&first_root_hash, &mut initial_keys)?;
        let second_items = bmt.get(&second_root_hash, &mut added_keys)?;

        for (key, value) in initial_keys.iter().zip(initial_data_values.iter()) {
            assert_eq!(first_items[&key[..]], Some(value.clone()));
        }
        for (key, value) in added_keys.iter().zip(added_data_values.iter()) {
            assert_eq!(second_items[&key[..]], Some(value.clone()));
        }
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_updates_an_existing_entry() -> BinaryMerkleTreeResult<()> {
        let seed = [0x25u8; 32];
        let path = generate_path(seed);

        let key = [0xAAu8; 32];
        let first_value = vec![0xBBu8];
        let second_value = vec![0xCCu8];

        let mut bmt = Tree::open(&path, 3)?;
        let first_root_hash = bmt.insert(None, &mut [&key], &mut vec![&first_value])?;
        let second_root_hash = bmt.insert(
            Some(&first_root_hash),
            &mut [&key],
            &mut vec![&second_value],
        )?;

        let first_item = bmt.get(&first_root_hash, &mut [&key])?;
        let second_item = bmt.get(&second_root_hash, &mut [&key])?;

        assert_eq!(first_item[&key[..]], Some(first_value));
        assert_eq!(second_item[&key[..]], Some(second_value));
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_updates_multiple_existing_entries() -> BinaryMerkleTreeResult<()> {
        let seed = [0x26u8; 32];
        let path = generate_path(seed);

        let seed = [0xEEu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        #[cfg(not(any(feature = "use_groestl")))]
        let prepare_initial = prepare_inserts(4096, &mut rng);
        #[cfg(feature = "use_groestl")]
        let prepare_initial = prepare_inserts(256, &mut rng);

        let initial_key_values = prepare_initial.0;
        let mut initial_keys = initial_key_values
            .iter()
            .map(|x| &x[..])
            .collect::<Vec<_>>();
        let initial_data_values = prepare_initial.1;
        let mut initial_data = initial_data_values.iter().collect::<Vec<_>>();

        let mut updated_data_values = vec![];
        let mut updated_data = vec![];
        let mut expected_updated_data_values = vec![];
        for i in 0..initial_key_values.len() {
            let num = vec![i as u8; 32];
            updated_data_values.push(num.clone());
            expected_updated_data_values.push(Some(num));
        }

        for i in 0..initial_key_values.len() {
            updated_data.push(updated_data_values[i].as_ref());
        }

        let mut bmt = Tree::open(&path, 160)?;
        let first_root_hash = bmt.insert(None, &mut initial_keys, &mut initial_data)?;
        let second_root_hash =
            bmt.insert(Some(&first_root_hash), &mut initial_keys, &mut updated_data)?;

        let initial_items = bmt.get(&first_root_hash, &mut initial_keys)?;
        let updated_items = bmt.get(&second_root_hash, &mut initial_keys)?;

        for (key, value) in initial_keys.iter().zip(initial_data.into_iter()) {
            assert_eq!(initial_items[&key[..]], Some(value.clone()));
        }
        for (key, value) in initial_keys.iter().zip(updated_data.into_iter()) {
            assert_eq!(updated_items[&key[..]], Some(value.clone()));
        }
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_does_not_panic_when_removing_a_nonexistent_node() -> BinaryMerkleTreeResult<()> {
        let seed = [0x27u8; 32];
        let path = generate_path(seed);

        let mut bmt = Tree::open(&path, 160)?;
        let missing_root_hash = vec![0x00u8];
        bmt.remove(&missing_root_hash)?;
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_removes_a_node() -> BinaryMerkleTreeResult<()> {
        let seed = [0x28u8; 32];
        let path = generate_path(seed);

        let key = [0x00u8; 32];
        let data = vec![0x01u8];

        let mut bmt = Tree::open(&path, 160)?;
        let root_hash = bmt.insert(None, &mut [&key], &mut vec![&data])?;

        let inserted_data = bmt.get(&root_hash, &mut [&key])?;

        assert_eq!(inserted_data[&key[..]], Some(data));

        bmt.remove(&root_hash)?;

        let retrieved_values = bmt.get(&root_hash, &mut [&key[..]])?;

        assert_eq!(retrieved_values[&key[..]], None);
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_removes_an_entire_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x29u8; 32];
        let path = generate_path(seed);

        let seed = [0xBBu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        #[cfg(not(any(feature = "use_groestl")))]
        let prepare = prepare_inserts(4096, &mut rng);
        #[cfg(feature = "use_groestl")]
        let prepare = prepare_inserts(256, &mut rng);

        let mut bmt = Tree::open(&path, 160)?;
        let key_values = prepare.0;
        let data_values = prepare.1;
        let mut keys = key_values.iter().map(|x| &x[..]).collect::<Vec<_>>();
        let mut data = data_values.iter().collect::<Vec<_>>();

        let root_hash = bmt.insert(None, &mut keys, &mut data)?;
        let inserted_items = bmt.get(&root_hash, &mut keys)?;

        for (key, value) in keys.iter().zip(data_values.iter()) {
            assert_eq!(inserted_items[&key[..]], Some(value.clone()));
        }

        bmt.remove(&root_hash)?;
        let removed_items = bmt.get(&root_hash, &mut keys)?;

        for key in keys.iter() {
            assert_eq!(removed_items[&key[..]], None);
        }
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_removes_an_old_root() -> BinaryMerkleTreeResult<()> {
        let seed = [0x30u8; 32];
        let path = generate_path(seed);

        let first_key = vec![0x00u8; 32];
        let first_data = vec![0x01u8];

        let mut bmt = Tree::open(&path, 160)?;
        let first_root_hash =
            bmt.insert(None, &mut vec![&first_key[..]], &mut vec![&first_data])?;

        let second_key = vec![0x02u8; 32];
        let second_data = vec![0x03u8];

        let second_root_hash = bmt.insert(
            Some(&first_root_hash),
            &mut vec![second_key.as_ref()],
            &mut vec![second_data.as_ref()],
        )?;
        bmt.remove(&first_root_hash)?;

        let retrieved_items = bmt.get(
            &second_root_hash,
            &mut vec![first_key.as_ref(), second_key.as_ref()],
        )?;
        assert_eq!(retrieved_items[&first_key[..]], Some(first_data));
        assert_eq!(retrieved_items[&second_key[..]], Some(second_data));
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_removes_a_small_old_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x31u8; 32];
        let path = generate_path(seed);

        let first_key = vec![0x00u8; 32];
        let second_key = vec![0x01u8; 32];
        let third_key = vec![0x02u8; 32];
        let fourth_key = vec![0x03u8; 32];

        let first_data = vec![0x04u8];
        let second_data = vec![0x05u8];
        let third_data = vec![0x06u8];
        let fourth_data = vec![0x07u8];

        let mut first_keys = vec![&first_key[..], &second_key[..]];
        let mut first_entries = vec![&first_data, &second_data];
        let mut bmt = Tree::open(&path, 160)?;
        let first_root_hash = bmt.insert(None, &mut first_keys, &mut first_entries)?;

        let mut second_keys = vec![&third_key[..], &fourth_key[..]];
        let mut second_entries = vec![&third_data, &fourth_data];
        let second_root_hash = bmt.insert(
            Some(&first_root_hash),
            &mut second_keys,
            &mut second_entries,
        )?;
        bmt.remove(&first_root_hash)?;

        let items = bmt.get(
            &second_root_hash,
            &mut vec![
                first_key.as_ref(),
                second_key.as_ref(),
                third_key.as_ref(),
                fourth_key.as_ref(),
            ],
        )?;
        for (key, value) in first_keys.iter().zip(first_entries.into_iter()) {
            assert_eq!(items[&key[..]], Some(value.clone()));
        }
        for (key, value) in second_keys.iter().zip(second_entries.into_iter()) {
            assert_eq!(items[&key[..]], Some(value.clone()));
        }
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_removes_an_old_large_root() -> BinaryMerkleTreeResult<()> {
        let seed = [0x32u8; 32];
        let path = generate_path(seed);

        let seed = [0xBAu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        let prepare_initial = prepare_inserts(16, &mut rng);
        let initial_key_values = prepare_initial.0;
        let initial_data_values = prepare_initial.1;
        let mut initial_keys = initial_key_values
            .iter()
            .map(|x| &x[..])
            .collect::<Vec<_>>();
        let mut initial_data = initial_data_values.iter().collect::<Vec<_>>();

        let mut bmt = Tree::open(&path, 160)?;
        let first_root_hash = bmt.insert(None, &mut initial_keys, &mut initial_data)?;

        let prepare_added = prepare_inserts(16, &mut rng);
        let added_key_values = prepare_added.0;
        let added_data_values = prepare_added.1;
        let mut added_keys = added_key_values.iter().map(|x| &x[..]).collect::<Vec<_>>();
        let mut added_data = added_data_values.iter().collect::<Vec<_>>();

        let second_root_hash =
            bmt.insert(Some(&first_root_hash), &mut added_keys, &mut added_data)?;

        bmt.remove(&first_root_hash)?;
        let initial_items = bmt.get(&second_root_hash, &mut initial_keys)?;
        let added_items = bmt.get(&second_root_hash, &mut added_keys)?;
        for (key, value) in initial_keys.iter().zip(initial_data.into_iter()) {
            assert_eq!(initial_items[&key[..]], Some(value.clone()));
        }
        for (key, value) in added_keys.iter().zip(added_data.into_iter()) {
            assert_eq!(added_items[&key[..]], Some(value.clone()));
        }
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_iterates_over_multiple_inserts_correctly() -> BinaryMerkleTreeResult<()> {
        let seed = [0x33u8; 32];
        let path = generate_path(seed);

        let seed = [0xEFu8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let mut bmt = Tree::open(&path, 160)?;

        #[cfg(not(any(feature = "use_groestl")))]
        iterate_inserts(8, 100, &mut rng, &mut bmt)?;
        #[cfg(feature = "use_groestl")]
        iterate_inserts(8, 10, &mut rng, &mut bmt)?;

        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_inserts_with_compressed_nodes_that_are_not_descendants() -> BinaryMerkleTreeResult<()> {
        let seed = [0x34u8; 32];
        let path = generate_path(seed);

        let mut bmt = Tree::open(&path, 160)?;

        let key_values = vec![
            vec![0x00u8; 32],
            vec![0x01u8; 32],
            vec![0x02u8; 32],
            vec![0x10u8; 32],
            vec![0x20u8; 32],
        ];
        let mut keys = key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let values = vec![
            vec![0x00u8],
            vec![0x01u8],
            vec![0x02u8],
            vec![0x03u8],
            vec![0x04u8],
        ];
        let mut data = values.iter().collect::<Vec<_>>();

        let first_root = bmt.insert(None, &mut keys[0..2], &mut data[0..2].to_vec())?;
        let second_root = bmt.insert(Some(&first_root), &mut keys[2..], &mut data[2..].to_vec())?;

        let items = bmt.get(&second_root, &mut keys)?;
        for (key, value) in keys.iter().zip(data.into_iter()) {
            assert_eq!(items[&key[..]], Some(value.clone()));
        }
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_inserts_with_compressed_nodes_that_are_descendants() -> BinaryMerkleTreeResult<()> {
        let seed = [0x35u8; 32];
        let path = generate_path(seed);

        let mut bmt = Tree::open(&path, 160)?;

        let key_values = vec![
            vec![0x10u8; 32],
            vec![0x11u8; 32],
            vec![0x00u8; 32],
            vec![0x01u8; 32],
            vec![0x02u8; 32],
        ];
        let mut keys = key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let values = vec![
            vec![0x00u8],
            vec![0x01u8],
            vec![0x02u8],
            vec![0x03u8],
            vec![0x04u8],
        ];
        let mut data = values.iter().collect::<Vec<_>>();

        let sorted_data = vec![
            vec![0x02u8],
            vec![0x03u8],
            vec![0x04u8],
            vec![0x00u8],
            vec![0x01u8],
        ];

        let first_root = bmt.insert(None, &mut keys[0..2], &mut data[0..2].to_vec())?;
        let second_root = bmt.insert(Some(&first_root), &mut keys[2..], &mut data[2..].to_vec())?;

        let items = bmt.get(&second_root, &mut keys)?;
        for (key, value) in keys.iter().zip(sorted_data.into_iter()) {
            assert_eq!(items[&key[..]], Some(value.clone()));
        }
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_correctly_iterates_removals() -> BinaryMerkleTreeResult<()> {
        let seed = [0x36u8; 32];
        let path = generate_path(seed);

        let seed = [0xA8u8; 32];
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let mut bmt = Tree::open(&path, 160)?;

        #[cfg(not(any(feature = "use_groestl")))]
        iterate_removals(8, 100, 1, &mut rng, &mut bmt)?;
        #[cfg(feature = "use_groestl")]
        iterate_removals(8, 10, 1, &mut rng, &mut bmt)?;
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_correctly_increments_a_leaf_reference_count() -> BinaryMerkleTreeResult<()> {
        let seed = [0x37u8; 32];
        let path = generate_path(seed);

        let mut bmt = Tree::open(&path, 160)?;

        let key = [0x00u8; 32];
        let data = vec![0x00u8];

        let first_root = bmt.insert(None, &mut [&key], &mut vec![data.as_ref()])?;
        let second_root = bmt.insert(Some(&first_root), &mut [&key], &mut vec![data.as_ref()])?;
        bmt.remove(&first_root)?;
        let item = bmt.get(&second_root, &mut [&key])?;
        assert_eq!(item[&key[..]], Some(data));
        tear_down(&path);
        Ok(())
    }

    fn generate_path(seed: [u8; 32]) -> PathBuf {
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let suffix = rng.gen_range(1000, 10000);
        let path_string = format!("Test_DB_{}", suffix);
        PathBuf::from(path_string)
    }

    fn tear_down(_path: &PathBuf) {
        #[cfg(feature = "use_rocksdb")]
        use std::fs::remove_dir_all;

        #[cfg(feature = "use_rocksdb")]
        remove_dir_all(&_path).unwrap();
    }

    fn prepare_inserts(num_entries: usize, rng: &mut StdRng) -> (Vec<[u8; 32]>, Vec<Vec<u8>>) {
        let mut keys = Vec::with_capacity(num_entries);
        let mut data = Vec::with_capacity(num_entries);
        for _ in 0..num_entries {
            let mut key_value = [0u8; 32];
            rng.fill(&mut key_value);
            keys.push(key_value);

            let mut data_value = [0u8; 32];
            rng.fill(data_value.as_mut());
            data.push(data_value.to_vec());
        }

        keys.sort();

        (keys, data)
    }

    fn iterate_inserts(
        entries_per_insert: usize,
        iterations: usize,
        rng: &mut StdRng,
        bmt: &mut Tree,
    ) -> BinaryMerkleTreeResult<(Vec<Option<[u8; 32]>>, Vec<Vec<[u8; 32]>>, Vec<Vec<Vec<u8>>>)>
    {
        let mut state_roots: Vec<Option<[u8; 32]>> = Vec::with_capacity(iterations);
        let mut key_groups = Vec::with_capacity(iterations);
        let mut data_groups = Vec::with_capacity(iterations);
        state_roots.push(None);

        for i in 0..iterations {
            let prepare = prepare_inserts(entries_per_insert, rng);
            let key_values = prepare.0;
            key_groups.push(key_values.clone());
            let data_values = prepare.1;
            data_groups.push(data_values.clone());

            let mut keys = key_values.iter().map(|x| &x[..]).collect::<Vec<_>>();
            let mut data = data_values.iter().collect::<Vec<_>>();

            let previous_state_root = &state_roots[i].clone();
            let previous_root;
            match previous_state_root {
                Some(r) => previous_root = Some(&r[..]),
                None => previous_root = None,
            }

            let new_root = bmt.insert(previous_root, &mut keys, &mut data)?;
            state_roots.push(Some(new_root.clone()));

            let retrieved_items = bmt.get(&new_root, &mut keys)?;
            for (key, value) in keys.iter().zip(data.into_iter()) {
                assert_eq!(retrieved_items[&key[..]], Some(value.clone()));
            }

            for j in 0..key_groups.len() {
                let mut key_block = Vec::with_capacity(key_groups[j].len());
                for k in 0..key_groups[j].len() {
                    key_block.push(key_groups[j][k].as_ref());
                }
                let items = bmt.get(&new_root, &mut key_block)?;
                for (key, value) in key_block.iter().zip(data_groups[j].iter()) {
                    assert_eq!(items[&key[..]], Some(value.clone()));
                }
            }
        }
        Ok((state_roots, key_groups, data_groups))
    }

    fn iterate_removals(
        entries_per_insert: usize,
        iterations: usize,
        removal_frequency: usize,
        rng: &mut StdRng,
        bmt: &mut Tree,
    ) -> BinaryMerkleTreeResult<()> {
        let inserts = iterate_inserts(entries_per_insert, iterations, rng, bmt)?;
        let state_roots = inserts.0;
        let key_groups = inserts.1;
        let data_groups = inserts.2;

        for i in 1..iterations {
            if i % removal_frequency == 0 {
                let root;
                if let Some(r) = state_roots[i].clone() {
                    root = r;
                } else {
                    panic!("state_roots[{}] is None", i);
                }
                bmt.remove(&root)?;
                for j in 0..iterations {
                    let mut keys = Vec::with_capacity(key_groups[i].len());
                    for k in 0..key_groups[i].len() {
                        keys.push(key_groups[i][k].as_ref());
                    }
                    let items = bmt.get(root.as_ref(), &mut keys)?;
                    if j % removal_frequency == 0 {
                        for key in keys.iter() {
                            assert_eq!(items[&key[..]], None);
                        }
                    } else {
                        for (key, value) in keys.iter().zip(data_groups[i].iter()) {
                            assert_eq!(items[&key[..]], Some(value.clone()));
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
