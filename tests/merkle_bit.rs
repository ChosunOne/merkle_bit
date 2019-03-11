#[cfg(test)]
pub mod integration_tests {
    use std::path::PathBuf;

    use rand::{Rng, SeedableRng};
    use rand::rngs::StdRng;

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
        let retrieved_value;
        let removed_retrieved_value;
        let data = vec![0xFFu8];
        let seed = [0x00u8; 32];
        let path = generate_path(seed);
        {
            let key = vec![0xAAu8];
            let mut values = vec![data.as_ref()];
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
                Ok(_) => {},
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
        assert_eq!(retrieved_value, vec![Some(data)]);
        assert_eq!(removed_retrieved_value, vec![None]);
        Ok(())
    }

    #[test]
    fn it_gets_an_item_out_of_a_simple_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x01u8; 32];
        let path = generate_path(seed);
        let key = vec![0xAAu8];
        let value = vec![0xFFu8];

        let mut bmt = Tree::open(&path, 160)?;
        let root = bmt.insert(None, &mut [&key[..]], &mut vec![&value])?;
        let result = bmt.get(&root, &mut vec![&key[..]])?;
        assert_eq!(result, vec![Some(vec![0xFFu8])]);
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_fails_to_get_from_empty_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x02u8; 32];
        let path = generate_path(seed);
        let key = vec![0x00u8];
        let root_key = vec![0x01u8];

        let bmt = Tree::open(&path, 160)?;
        let items = bmt.get(&root_key, &mut vec![&key[..]])?;
        let expected_items = vec![None];
        assert_eq!(items, expected_items);
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_fails_to_get_a_nonexistent_item() -> BinaryMerkleTreeResult<()> {
        let seed = [0x03u8; 32];
        let path = generate_path(seed);
        let key = vec![0xAAu8];
        let value = vec![0xFFu8];

        let mut bmt = Tree::open(&path, 160)?;
        let root = bmt.insert(None, &mut [&key[..]], &mut vec![&value])?;

        let nonexistent_key = vec![0xAB];
        let items = bmt.get(&root, &mut vec![&nonexistent_key[..]])?;
        let expected_items = vec![None];
        assert_eq!(items, expected_items);
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
            keys.push(vec![i << 5]);
            values.push(vec![i]);
        }
        let mut get_keys = keys.iter().map(|x| x.as_slice()).collect::<Vec<_>>();

        let mut bmt = Tree::open(&path, 3)?;
        let mut insert_values = values.iter().collect::<Vec<_>>();
        let root_hash = bmt.insert(None, &mut get_keys, &mut insert_values)?;

        let items = bmt.get(&root_hash, &mut get_keys)?;
        let mut expected_items = vec![];
        for value in values {
            expected_items.push(Some(value));
        }
        assert_eq!(items, expected_items);
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_gets_items_from_a_small_unbalanced_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x05u8; 32];
        let path = generate_path(seed);
        let mut keys: Vec<Vec<u8>> = Vec::with_capacity(7);
        let mut values: Vec<Vec<u8>> = Vec::with_capacity(7);
        for i in 0..7 {
            keys.push(vec![i << 5]);
            values.push(vec![i]);
        }
        let mut get_keys = keys.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let mut insert_values = values.iter().collect::<Vec<_>>();
        let mut bmt = Tree::open(&path, 3)?;

        let root_hash = bmt.insert(None, &mut get_keys, &mut insert_values)?;
        let items = bmt.get(&root_hash, &mut get_keys)?;
        let mut expected_items = vec![];
        for value in values {
            expected_items.push(Some(value));
        }
        assert_eq!(items, expected_items);
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_gets_items_from_a_medium_balanced_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x06u8; 32];
        let path = generate_path(seed);

        let num_leaves = 256;
        let mut keys: Vec<Vec<u8>> = Vec::with_capacity(num_leaves);
        let mut values: Vec<Vec<u8>> = Vec::with_capacity(num_leaves);
        for i in 0..num_leaves {
            keys.push(vec![i as u8]);
            values.push(vec![i as u8]);
        }

        let mut get_keys = keys.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let mut insert_values = values.iter().collect::<Vec<_>>();

        let mut bmt = Tree::open(&path, 8)?;
        let root_hash = bmt.insert(None, &mut get_keys, &mut insert_values)?;

        let items = bmt.get(&root_hash, &mut get_keys)?;
        let mut expected_items = vec![];
        for value in values {
            expected_items.push(Some(value));
        }
        assert_eq!(items, expected_items);
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_gets_items_from_a_medium_unbalanced_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x07u8; 32];
        let path = generate_path(seed);
        let num_leaves = 255;
        let mut keys: Vec<Vec<u8>> = Vec::with_capacity(num_leaves);
        let mut values: Vec<Vec<u8>> = Vec::with_capacity(num_leaves);
        for i in 0..num_leaves {
            keys.push(vec![i as u8]);
            values.push(vec![i as u8]);
        }

        let mut get_keys = keys.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let mut insert_values = values.iter().collect::<Vec<_>>();

        let mut bmt = Tree::open(&path, 8)?;
        let root_hash = bmt.insert(None, &mut get_keys, &mut insert_values)?;

        let items = bmt.get(&root_hash, &mut get_keys)?;
        let mut expected_items = vec![];
        for value in values {
            expected_items.push(Some(value));
        }
        assert_eq!(items, expected_items);
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

        let mut keys: Vec<Vec<u8>> = Vec::with_capacity(num_leaves);
        let mut values: Vec<Vec<u8>> = Vec::with_capacity(num_leaves);
        for i in 0..num_leaves {
            keys.push(vec![(i >> 8) as u8, (i & 0xFF) as u8]);
            values.push(vec![(i >> 8) as u8, (i & 0xFF) as u8]);
        }

        let mut get_keys = keys.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let mut insert_values = values.iter().collect::<Vec<_>>();

        let mut bmt = Tree::open(&path, 16)?;
        let root_hash = bmt.insert(None, &mut get_keys, &mut insert_values)?;

        let items = bmt.get(&root_hash, &mut get_keys)?;
        let mut expected_items = vec![];
        for value in values {
            expected_items.push(Some(value));
        }
        assert_eq!(items, expected_items);
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
        let mut keys: Vec<Vec<u8>> = Vec::with_capacity(num_leaves);
        let mut values: Vec<Vec<u8>> = Vec::with_capacity(num_leaves);
        for i in 0..num_leaves {
            keys.push(vec![(i >> 8) as u8, (i & 0xFF) as u8]);
            values.push(vec![(i >> 8) as u8, (i & 0xFF) as u8]);
        }

        let mut get_keys = keys.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let mut insert_values = values.iter().collect::<Vec<_>>();

        let mut bmt = Tree::open(&path, 16)?;
        let root_hash = bmt.insert(None, &mut get_keys, &mut insert_values)?;

        let items = bmt.get(&root_hash, &mut get_keys)?;
        let mut expected_items = vec![];
        for value in values {
            expected_items.push(Some(value));
        }
        assert_eq!(items, expected_items);
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
        let pop_key_d = vec![0x60u8]; // 0110_0000   96 (Dec)
        let pop_key_e = vec![0x68u8]; // 0110_1000  104 (Dec)
        let pop_key_i = vec![0x80u8]; // 1000_0000  128 (Dec)
        let pop_key_o = vec![0xF0u8]; // 1111_0000  240 (Dec)

        let mut populated_keys = [&pop_key_d[..], &pop_key_e[..], &pop_key_i[..], &pop_key_o[..]];

        let pop_value_d = vec![0x01u8];
        let pop_value_e = vec![0x02u8];
        let pop_value_i = vec![0x03u8];
        let pop_value_o = vec![0x04u8];

        let mut populated_values = vec![&pop_value_d, &pop_value_e, &pop_value_i, &pop_value_o];

        let mut bmt = Tree::open(&path, 5)?;
        let root_node = bmt.insert(None, &mut populated_keys, &mut populated_values)?;

        let key_a = vec![0x00u8]; // 0000_0000     0 (Dec)
        let key_b = vec![0x40u8]; // 0100_0000    64 (Dec)
        let key_c = vec![0x41u8]; // 0100_0001    65 (Dec)
        let key_f = vec![0x70u8]; // 0111_0000   112 (Dec)
        let key_g = vec![0x71u8]; // 0111_0001   113 (Dec)
        let key_h = vec![0x72u8]; // 0111_0010   114 (Dec)
        let key_j = vec![0xC0u8]; // 1100_0000   192 (Dec)
        let key_k = vec![0xC1u8]; // 1100_0001   193 (Dec)
        let key_l = vec![0xE0u8]; // 1110_0000   224 (Dec)
        let key_m = vec![0xE1u8]; // 1110_0001   225 (Dec)
        let key_n = vec![0xE2u8]; // 1110_0010   226 (Dec)
        let key_p = vec![0xF8u8]; // 1111_1000   248 (Dec)

        let mut keys = vec![
            &key_a[..], &key_b[..], &key_c[..], &pop_key_d[..],
            &pop_key_e[..], &key_f[..], &key_g[..], &key_h[..],
            &pop_key_i[..], &key_j[..], &key_k[..], &key_l[..],
            &key_m[..], &key_n[..], &pop_key_o[..], &key_p[..]];


        let items = bmt.get(&root_node, &mut keys)?;
        let expected_items = vec![
            None, None, None, Some(pop_value_d),
            Some(pop_value_e), None, None, None,
            Some(pop_value_i), None, None, None,
            None, None, Some(pop_value_o), None];
        assert_eq!(items, expected_items);
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_returns_the_same_number_of_values_as_keys() -> BinaryMerkleTreeResult<()> {
        let seed = [0x11u8; 32];
        let path = generate_path(seed);

        let initial_key = vec![0x00u8];
        let initial_value = vec![0xFFu8];

        let mut keys = Vec::with_capacity(256);
        for i in 0..256 {
            keys.push(vec![i as u8]);
        }

        let mut get_keys = keys.iter().map(|x| x.as_slice()).collect::<Vec<_>>();

        let mut bmt = Tree::open(&path, 3)?;
        let root_node = bmt.insert(None, &mut [&initial_key], &mut vec![&initial_value])?;

        let items = bmt.get(&root_node, &mut get_keys)?;
        let mut expected_items = vec![];
        for i in 0..256 {
            if i == 0 {
                expected_items.push(Some(vec![0xFF]));
            } else {
                expected_items.push(None);
            }
        }
        assert_eq!(items.len(), 256);
        assert_eq!(items, expected_items);
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_inserts_two_leaf_nodes_into_empty_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x12u8; 32];
        let path = generate_path(seed);

        let key_values = vec![
            vec![0x00u8],
            vec![0x01u8]
        ];
        let mut keys = key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let data_values = vec![
            vec![0x02u8],
            vec![0x03u8]
        ];
        let mut data = data_values.iter().collect::<Vec<_>>();

        let mut bmt = Tree::open(&path, 3)?;
        let root_hash = bmt.insert(None, &mut keys, &mut data)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        let expected_items = vec![Some(vec![0x02u8]), Some(vec![0x03u8])];
        assert_eq!(items, expected_items);
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_inserts_two_leaf_nodes_into_empty_tree_with_first_bit_split() -> BinaryMerkleTreeResult<()> {
        let seed = [0x13u8; 32];
        let path = generate_path(seed);

        let key_values = vec![
            vec![0x00u8],
            vec![0x80u8]
        ];
        let mut keys = key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let data_values = vec![
            vec![0x02u8],
            vec![0x03u8]
        ];
        let mut data = data_values.iter().collect::<Vec<_>>();

        let mut bmt = Tree::open(&path, 3)?;
        let root_hash = bmt.insert(None, &mut keys, &mut data)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        let expected_items = vec![Some(vec![0x02u8]), Some(vec![0x03u8])];
        assert_eq!(items, expected_items);
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_inserts_a_leaf_node_into_empty_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x14u8; 32];
        let path = generate_path(seed);

        let key = vec![0xAAu8];
        let data = vec![0xBBu8];

        let mut bmt = Tree::open(&path, 3)?;
        let new_root_hash = bmt.insert(None, &mut [&key[..]], &mut vec![data.as_ref()])?;
        let items = bmt.get(&new_root_hash, &mut vec![&key[..]])?;
        let expected_items = vec![Some(vec![0xBBu8])];
        assert_eq!(items, expected_items);
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_inserts_multiple_leaf_nodes_into_empty_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x15u8; 32];
        let path = generate_path(seed);

        let key_values = vec![
            vec![0xAAu8],  // 1010_1010
            vec![0xBBu8],  // 1011_1011
            vec![0xCCu8]]; // 1100_1100
        let mut keys = key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let data_values = vec![vec![0xDDu8], vec![0xEEu8], vec![0xFFu8]];
        let mut data = data_values.iter().collect::<Vec<_>>();

        let mut bmt = Tree::open(&path, 3)?;
        let root_hash = bmt.insert(None, &mut keys, &mut data)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        let expected_items = vec![Some(vec![0xDDu8]), Some(vec![0xEEu8]), Some(vec![0xFFu8])];
        assert_eq!(items, expected_items);
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
        let mut keys = key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let data_values = prepare.1;
        let mut data = data_values.iter().collect::<Vec<_>>();
        let expected_items = prepare.2;

        let mut bmt = Tree::open(&path, 16)?;
        let root_hash = bmt.insert(None, &mut keys, &mut data)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        assert_eq!(items, expected_items);
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
        let mut keys = key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let data_values = prepare.1;
        let mut data = data_values.iter().collect::<Vec<_>>();
        let expected_items = prepare.2;

        let mut bmt = Tree::open(&path, 16)?;
        let root_hash = bmt.insert(None, &mut keys, &mut data)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        assert_eq!(items, expected_items);
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
        let mut keys = key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let data_values = prepare.1;
        let mut data = data_values.iter().collect::<Vec<_>>();
        let expected_items = prepare.2;

        let mut bmt = Tree::open(&path, 16)?;
        let root_hash = bmt.insert(None, &mut keys, &mut data)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        assert_eq!(items, expected_items);
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
        let mut keys = key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let data_values = prepare.1;
        let mut data = data_values.iter().collect::<Vec<_>>();

        let expected_items = prepare.2;

        let mut bmt = Tree::open(&path, 16)?;
        let root_hash = bmt.insert(None, &mut keys, &mut data)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        assert_eq!(items, expected_items);
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
        let mut keys = key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let data_values = prepare.1;
        let mut data = data_values.iter().collect::<Vec<_>>();
        let expected_items = prepare.2;

        let mut bmt = Tree::open(&path, 16)?;
        let root_hash = bmt.insert(None, &mut keys, &mut data)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        assert_eq!(items, expected_items);
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
        let mut keys = key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let data_values = prepare.1;
        let mut data = data_values.iter().collect::<Vec<_>>();
        let expected_items = prepare.2;

        let mut bmt = Tree::open(&path, 16)?;
        let root_hash = bmt.insert(None, &mut keys, &mut data)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        assert_eq!(items, expected_items);
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_inserts_a_leaf_node_into_a_tree_with_one_item() -> BinaryMerkleTreeResult<()> {
        let seed = [0x22u8; 32];
        let path = generate_path(seed);

        let first_key = vec![0xAAu8];
        let first_data = vec![0xBBu8];

        let second_key = vec![0xCCu8];
        let second_data = vec![0xDDu8];

        let mut bmt = Tree::open(&path, 3)?;
        let new_root_hash = bmt.insert(None, &mut vec![first_key.as_ref()], &mut vec![first_data.as_ref()])?;
        let second_root_hash = bmt.insert(Some(&new_root_hash), &mut vec![second_key.as_ref()], &mut vec![second_data.as_ref()])?;

        let items = bmt.get(&second_root_hash, &mut vec![first_key.as_ref(), second_key.as_ref()])?;
        let expected_items = vec![Some(vec![0xBBu8]), Some(vec![0xDDu8])];
        assert_eq!(items, expected_items);
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_inserts_multiple_leaf_nodes_into_a_small_tree_with_existing_items() -> BinaryMerkleTreeResult<()> {
        let seed = [0x23u8; 32];
        let path = generate_path(seed);

        let seed = [0x4d, 0x1b, 0xf8, 0xad, 0x2d, 0x5d, 0x2e, 0xcb, 0x59, 0x75, 0xc4, 0xb9,
            0x4d, 0xf9, 0xab, 0x5e, 0xf5, 0x12, 0xd4, 0x5c, 0x3d, 0xa0, 0x73, 0x4b,
            0x65, 0x5e, 0xc3, 0x82, 0xcb, 0x6c, 0xc0, 0x66];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        let num_inserts = 2;
        let prepare_initial = prepare_inserts(num_inserts, &mut rng);
        let initial_key_values = prepare_initial.0;
        let mut initial_keys = initial_key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let initial_data_values = prepare_initial.1;
        let mut initial_data = initial_data_values.iter().collect::<Vec<_>>();

        let mut bmt = Tree::open(&path, 160)?;
        let first_root_hash = bmt.insert(None, &mut initial_keys, &mut initial_data)?;

        let prepare_added = prepare_inserts(num_inserts, &mut rng);
        let added_key_values = prepare_added.0;
        let mut added_keys = added_key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let added_data_values = prepare_added.1;
        let mut added_data = added_data_values.iter().collect::<Vec<_>>();

        let second_root_hash = bmt.insert(Some(&first_root_hash), &mut added_keys, &mut added_data)?;

        let first_items = bmt.get(&first_root_hash, &mut initial_keys)?;
        let second_items = bmt.get(&second_root_hash, &mut added_keys)?;

        let expected_initial_items = prepare_initial.2;
        let expected_added_items = prepare_added.2;

        assert_eq!(first_items, expected_initial_items);
        assert_eq!(second_items, expected_added_items);
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_inserts_multiple_leaf_nodes_into_a_tree_with_existing_items() -> BinaryMerkleTreeResult<()> {
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
        let mut initial_keys = initial_key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let initial_data_values = prepare_initial.1;
        let mut initial_data = initial_data_values.iter().collect::<Vec<_>>();

        let mut bmt = Tree::open(&path, 160)?;
        let first_root_hash = bmt.insert(None, &mut initial_keys, &mut initial_data)?;

        let prepare_added = prepare_inserts(num_inserts, &mut rng);
        let added_key_values = prepare_added.0;
        let mut added_keys = added_key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let added_data_values = prepare_added.1;
        let mut added_data = added_data_values.iter().collect::<Vec<_>>();

        let second_root_hash = bmt.insert(Some(&first_root_hash), &mut added_keys, &mut added_data)?;

        let first_items = bmt.get(&first_root_hash, &mut initial_keys)?;
        let second_items = bmt.get(&second_root_hash, &mut added_keys)?;

        let expected_initial_items = prepare_initial.2;
        let expected_added_items = prepare_added.2;

        assert_eq!(first_items, expected_initial_items);
        assert_eq!(second_items, expected_added_items);
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_updates_an_existing_entry() -> BinaryMerkleTreeResult<()> {
        let seed = [0x25u8; 32];
        let path = generate_path(seed);

        let key = vec![0xAAu8];
        let first_value = vec![0xBBu8];
        let second_value = vec![0xCCu8];

        let mut bmt = Tree::open(&path, 3)?;
        let first_root_hash = bmt.insert(None, &mut vec![key.as_ref()], &mut vec![first_value.as_ref()])?;
        let second_root_hash = bmt.insert(Some(&first_root_hash), &mut vec![key.as_ref()], &mut vec![second_value.as_ref()])?;

        let first_item = bmt.get(&first_root_hash, &mut vec![key.as_ref()])?;
        let expected_first_item = vec![Some(first_value.clone())];

        let second_item = bmt.get(&second_root_hash, &mut vec![key.as_ref()])?;
        let expected_second_item = vec![Some(second_value.clone())];

        assert_eq!(first_item, expected_first_item);
        assert_eq!(second_item, expected_second_item);
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
        let mut initial_keys = initial_key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
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
        let second_root_hash = bmt.insert(Some(&first_root_hash), &mut initial_keys, &mut updated_data)?;

        let initial_items = bmt.get(&first_root_hash, &mut initial_keys)?;
        let updated_items = bmt.get(&second_root_hash, &mut initial_keys)?;

        let expected_initial_items = prepare_initial.2;
        assert_eq!(initial_items, expected_initial_items);
        assert_eq!(updated_items, expected_updated_data_values);
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

        let key = vec![0x00];
        let data = vec![0x01];

        let mut bmt = Tree::open(&path, 160)?;
        let root_hash = bmt.insert(None, &mut vec![key.as_ref()], &mut vec![data.as_ref()])?;

        let inserted_data = bmt.get(&root_hash, &mut vec![key.as_ref()])?;
        let expected_inserted_data = vec![Some(vec![0x01u8])];
        assert_eq!(inserted_data, expected_inserted_data);

        bmt.remove(&root_hash)?;

        let retrieved_values = bmt.get(&root_hash, &mut vec![key.as_ref()])?;
        let expected_retrieved_values = vec![None];
        assert_eq!(retrieved_values, expected_retrieved_values);
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
        let mut keys = key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let mut data = data_values.iter().collect::<Vec<_>>();

        let root_hash = bmt.insert(None, &mut keys, &mut data)?;
        let expected_inserted_items = prepare.2;
        let inserted_items = bmt.get(&root_hash, &mut keys)?;
        assert_eq!(inserted_items, expected_inserted_items);

        bmt.remove(&root_hash)?;
        let removed_items = bmt.get(&root_hash, &mut keys)?;
        let mut expected_removed_items = vec![];
        for _ in 0..keys.len() {
            expected_removed_items.push(None);
        }
        assert_eq!(removed_items, expected_removed_items);
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_removes_an_old_root() -> BinaryMerkleTreeResult<()> {
        let seed = [0x30u8; 32];
        let path = generate_path(seed);

        let first_key = vec![0x00u8];
        let first_data = vec![0x01u8];

        let mut bmt = Tree::open(&path, 160)?;
        let first_root_hash = bmt.insert(None, &mut vec![first_key.as_ref()], &mut vec![first_data.as_ref()])?;

        let second_key = vec![0x02u8];
        let second_data = vec![0x03u8];

        let second_root_hash = bmt.insert(Some(&first_root_hash), &mut vec![second_key.as_ref()], &mut vec![second_data.as_ref()])?;
        bmt.remove(&first_root_hash)?;

        let retrieved_items = bmt.get(&second_root_hash, &mut vec![first_key.as_ref(), second_key.as_ref()])?;
        let expected_retrieved_items = vec![Some(vec![0x01u8]), Some(vec![0x03u8])];
        assert_eq!(retrieved_items, expected_retrieved_items);
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_removes_a_small_old_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x31u8; 32];
        let path = generate_path(seed);

        let first_key = vec![0x00u8];
        let second_key = vec![0x01u8];
        let third_key = vec![0x02u8];
        let fourth_key = vec![0x03u8];

        let first_data = vec![0x04u8];
        let second_data = vec![0x05u8];
        let third_data = vec![0x06u8];
        let fourth_data = vec![0x07u8];

        let mut first_keys = vec![first_key.as_ref(), second_key.as_ref()];
        let mut first_entries = vec![first_data.as_ref(), second_data.as_ref()];
        let mut bmt = Tree::open(&path, 160)?;
        let first_root_hash = bmt.insert(None, &mut first_keys, &mut first_entries)?;

        let mut second_keys = vec![third_key.as_ref(), fourth_key.as_ref()];
        let mut second_entries = vec![third_data.as_ref(), fourth_data.as_ref()];
        let second_root_hash = bmt.insert(Some(&first_root_hash), &mut second_keys, &mut second_entries)?;
        bmt.remove(&first_root_hash)?;

        let items = bmt.get(&second_root_hash, &mut vec![first_key.as_ref(), second_key.as_ref(), third_key.as_ref(), fourth_key.as_ref()])?;
        let expected_items = vec![Some(first_data.clone()), Some(second_data.clone()), Some(third_data.clone()), Some(fourth_data.clone())];
        assert_eq!(items, expected_items);
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
        let mut initial_keys = initial_key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let mut initial_data = initial_data_values.iter().collect::<Vec<_>>();

        let mut bmt = Tree::open(&path, 160)?;
        let first_root_hash = bmt.insert(None, &mut initial_keys, &mut initial_data)?;

        let prepare_added = prepare_inserts(16, &mut rng);
        let added_key_values = prepare_added.0;
        let added_data_values = prepare_added.1;
        let mut added_keys = added_key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let mut added_data = added_data_values.iter().collect::<Vec<_>>();

        let second_root_hash = bmt.insert(Some(&first_root_hash), &mut added_keys, &mut added_data)?;

        let combined_size = initial_key_values.len() + added_key_values.len();
        let mut combined_keys = Vec::with_capacity(combined_size);
        let mut combined_expected_items = Vec::with_capacity(combined_size);
        let mut i = 0;
        let mut j = 0;
        for _ in 0..combined_size {
            if i == initial_key_values.len() {
                if j < added_key_values.len() {
                    combined_keys.push(added_key_values[j].as_ref());
                    combined_expected_items.push(prepare_added.2[j].clone());
                    j += 1;
                    continue;
                }
                continue;
            } else if j == added_key_values.len() {
                if i < initial_key_values.len() {
                    combined_keys.push(initial_key_values[i].as_ref());
                    combined_expected_items.push(prepare_initial.2[i].clone());
                    i += 1;
                    continue;
                }
                continue;
            }

            if i < initial_key_values.len() && initial_key_values[i] < added_key_values[j] {
                combined_keys.push(initial_key_values[i].as_ref());
                combined_expected_items.push(prepare_initial.2[i].clone());
                i += 1;
            } else if j < added_key_values.len() {
                combined_keys.push(added_key_values[j].as_ref());
                combined_expected_items.push(prepare_added.2[j].clone());
                j += 1;
            }
        }

        bmt.remove(&first_root_hash)?;
        let items = bmt.get(&second_root_hash, &mut combined_keys)?;
        assert_eq!(items, combined_expected_items);
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

        let key_values = vec![vec![0x00u8], vec![0x01u8], vec![0x02u8], vec![0x10u8], vec![0x20u8]];
        let mut keys = key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let values = vec![vec![0x00u8], vec![0x01u8], vec![0x02u8], vec![0x03u8], vec![0x04u8]];
        let mut data = values.iter().collect::<Vec<_>>();

        let first_root = bmt.insert(None, &mut keys[0..2], &mut data[0..2].to_vec())?;
        let second_root = bmt.insert(Some(&first_root), &mut keys[2..], &mut data[2..].to_vec())?;

        let items = bmt.get(&second_root, &mut keys)?;
        let mut expected_items = Vec::with_capacity(values.len());
        for i in 0..values.len() {
            expected_items.push(Some(values[i].clone()));
        }

        assert_eq!(items, expected_items);
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_inserts_with_compressed_nodes_that_are_descendants() -> BinaryMerkleTreeResult<()> {
        let seed = [0x35u8; 32];
        let path = generate_path(seed);

        let mut bmt = Tree::open(&path, 160)?;

        let key_values = vec![vec![0x10u8], vec![0x11u8], vec![0x00u8], vec![0x01u8], vec![0x02u8]];
        let mut keys = key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
        let values = vec![vec![0x00u8], vec![0x01u8], vec![0x02u8], vec![0x03u8], vec![0x04u8]];
        let mut data = values.iter().collect::<Vec<_>>();

        let first_root = bmt.insert(None, &mut keys[0..2], &mut data[0..2].to_vec())?;
        let second_root = bmt.insert(Some(&first_root), &mut keys[2..], &mut data[2..].to_vec())?;

        keys.sort();

        let items = bmt.get(&second_root, &mut keys)?;
        let expected_items = vec![Some(vec![0x02u8]), Some(vec![0x03u8]), Some(vec![0x04u8]), Some(vec![0x00u8]), Some(vec![0x01u8])];
        assert_eq!(items, expected_items);
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

        let key = vec![0x00u8];
        let data = vec![0x00u8];

        let first_root = bmt.insert(None, &mut vec![key.as_ref()], &mut vec![data.as_ref()])?;
        let second_root = bmt.insert(Some(&first_root), &mut vec![key.as_ref()], &mut vec![data.as_ref()])?;
        bmt.remove(&first_root)?;
        let item = bmt.get(&second_root, &mut vec![key.as_ref()])?;
        let expected_item = vec![Some(vec![0x00u8])];
        assert_eq!(item, expected_item);
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

    fn iterate_inserts(entries_per_insert: usize,
                       iterations: usize,
                       rng: &mut StdRng,
                       bmt: &mut Tree) -> BinaryMerkleTreeResult<(Vec<Option<Vec<u8>>>, Vec<Vec<Vec<u8>>>, Vec<Vec<Option<Vec<u8>>>>)> {
        let mut state_roots: Vec<Option<Vec<u8>>> = Vec::with_capacity(iterations);
        let mut key_groups = Vec::with_capacity(iterations);
        let mut data_groups = Vec::with_capacity(iterations);
        state_roots.push(None);

        for i in 0..iterations {
            let prepare = prepare_inserts(entries_per_insert, rng);
            let key_values = prepare.0;
            key_groups.push(key_values.clone());
            let data_values = prepare.1;
            let expected_data_values = prepare.2;
            data_groups.push(expected_data_values.clone());

            let mut keys = key_values.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
            let mut data = data_values.iter().collect::<Vec<_>>();

            let previous_state_root = &state_roots[i].clone();
            let previous_root;
            match previous_state_root {
                Some(r) => previous_root = Some(r.as_slice()),
                None => previous_root = None
            }

            let new_root = bmt.insert(previous_root, &mut keys, &mut data)?;
            state_roots.push(Some(new_root.clone()));


            let retrieved_items = bmt.get(&new_root, &mut keys)?;
            assert_eq!(retrieved_items, expected_data_values);


            for j in 0..key_groups.len() {
                let mut key_block = Vec::with_capacity(key_groups[j].len());
                for k in 0..key_groups[j].len() {
                    key_block.push(key_groups[j][k].as_ref());
                }
                let items = bmt.get(&new_root, &mut key_block)?;
                assert_eq!(items, data_groups[j]);
            }
        }
        Ok((state_roots, key_groups, data_groups))
    }

    fn iterate_removals(entries_per_insert: usize,
                        iterations: usize,
                        removal_frequency: usize,
                        rng: &mut StdRng,
                        bmt: &mut Tree) -> BinaryMerkleTreeResult<()> {
        let inserts = iterate_inserts(entries_per_insert, iterations, rng, bmt)?;
        let state_roots = inserts.0;
        let key_groups = inserts.1;
        let data_groups = inserts.2;

        for i in 1..iterations {
            if i % removal_frequency == 0 {
                let root;
                if let Some(r) = state_roots[i].clone() {
                    root = r.clone();
                } else {
                    panic!("state_roots[{}] is None", i);
                }
                bmt.remove(root.as_ref())?;
                for j in 0..iterations {
                    let mut keys = Vec::with_capacity(key_groups[i].len());
                    for k in 0..key_groups[i].len() {
                        keys.push(key_groups[i][k].as_ref());
                    }
                    let items = bmt.get(root.as_ref(), &mut keys)?;
                    let mut expected_items;
                    if j % removal_frequency == 0 {
                        expected_items = Vec::with_capacity(key_groups[i].len());
                        for _ in 0..key_groups[i].len() {
                            expected_items.push(None);
                        }
                    } else {
                        expected_items = data_groups[i].clone();
                    }
                    assert_eq!(items, expected_items);
                }
            }
        }
        Ok(())
    }
}
