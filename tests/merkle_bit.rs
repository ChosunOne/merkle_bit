#[cfg(test)]
pub mod integration_tests {
    const KEY_LEN: usize = 32;
    use std::path::PathBuf;

    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};
    use starling::Array;

    #[cfg(not(any(feature = "rocksdb")))]
    use starling::hash_tree::HashTree;
    use starling::merkle_bit::BinaryMerkleTreeResult;
    #[cfg(feature = "rocksdb")]
    use starling::rocks_tree::RocksTree;
    use starling::traits::Exception;

    #[cfg(feature = "rocksdb")]
    type Tree = RocksTree;

    #[cfg(not(any(feature = "rocksdb")))]
    type Tree = HashTree;

    macro_rules! test_key_size {
        ($func_name: ident, $key_size: expr, $seed: expr, $num_entries: expr, $num_entries_groestl: expr) => {
            #[test]
            fn $func_name() -> BinaryMerkleTreeResult<()> {
                #[cfg(feature = "rocksdb")]
                type Tree = RocksTree<$key_size>;

                #[cfg(not(any(feature = "rocksdb")))]
                type Tree = HashTree<$key_size>;

                let seed = $seed;
                let path = generate_path(seed);
                let mut rng: StdRng = SeedableRng::from_seed(seed);

                #[cfg(not(feature = "groestl"))]
                let num_entries = $num_entries;
                #[cfg(feature = "groestl")]
                let num_entries = $num_entries_groestl;

                let mut keys = Vec::with_capacity(num_entries);
                let mut values = Vec::with_capacity(num_entries);
                for _ in 0..num_entries {
                    let mut key_value = [0u8; $key_size];
                    rng.fill(&mut key_value);
                    keys.push(key_value.into());

                    let data_value: Vec<u8> = (0..$key_size).map(|_| rng.gen()).collect();
                    values.push(data_value);
                }

                keys.sort();

                let mut bmt = Tree::open(&path, 160)?;

                let root = bmt.insert(None, &mut keys, &values)?;

                let retrieved = bmt.get(&root, &mut keys)?;

                tear_down(&path);
                for (&key, value) in keys.iter().zip(values) {
                    assert_eq!(retrieved[&key], Some(value));
                }

                Ok(())
            }
        };
    }

    #[test]
    #[cfg(feature = "serde")]
    fn it_works_with_a_real_database() -> BinaryMerkleTreeResult<()> {
        let seed = [0x00u8; KEY_LEN];
        let path = generate_path(seed);
        let key = [0xAAu8; KEY_LEN];
        let retrieved_value;
        let removed_retrieved_value;
        let data = vec![0xFFu8];
        {
            let values = vec![data.clone()];
            let mut tree = Tree::open(&path, 160)?;
            let root;
            match tree.insert(None, &mut [key.into()], &values) {
                Ok(r) => root = r,
                Err(e) => {
                    drop(tree);
                    tear_down(&path);
                    panic!("{:?}", &e.to_string());
                }
            }
            match tree.get(&root, &mut [key.into()]) {
                Ok(v) => retrieved_value = v,
                Err(e) => {
                    drop(tree);
                    tear_down(&path);
                    panic!("{:?}", &e.to_string());
                }
            }
            match tree.remove(&root) {
                Ok(_) => {}
                Err(e) => {
                    drop(tree);
                    tear_down(&path);
                    panic!("{:?}", &e.to_string());
                }
            }
            match tree.get(&root, &mut [key.into()]) {
                Ok(v) => removed_retrieved_value = v,
                Err(e) => {
                    drop(tree);
                    tear_down(&path);
                    panic!("{:?}", &e.to_string());
                }
            }
        }
        tear_down(&path);
        assert_eq!(retrieved_value[&key.into()], Some(data));
        assert_eq!(removed_retrieved_value[&key.into()], None);
        Ok(())
    }

    #[test]
    fn it_gets_an_item_out_of_a_simple_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x01u8; KEY_LEN];
        let path = generate_path(seed);
        #[cfg(not(any(feature = "serde")))]
        let key = [0xAAu8; KEY_LEN];
        #[cfg(feature = "serde")]
        let key = [0xAAu8; KEY_LEN].into();
        let value = vec![0xFFu8];

        let mut bmt = Tree::open(&path, 160)?;
        let root = bmt.insert(None, &mut [key], &vec![value])?;
        let result = bmt.get(&root, &mut vec![key])?;
        tear_down(&path);
        assert_eq!(result[&key], Some(vec![0xFFu8]));
        Ok(())
    }

    #[test]
    fn it_fails_to_get_from_empty_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x02u8; KEY_LEN];
        let path = generate_path(seed);
        #[cfg(not(any(feature = "serde")))]
        let key = [0x00u8; KEY_LEN];
        #[cfg(feature = "serde")]
        let key = [0x00_u8; KEY_LEN].into();
        #[cfg(not(any(feature = "serde")))]
        let root_key = [0x01u8; KEY_LEN];
        #[cfg(feature = "serde")]
        let root_key = [0x01u8; KEY_LEN].into();

        let bmt = Tree::open(&path, 160)?;
        let items = bmt.get(&root_key, &mut [key])?;
        let expected_item = None;
        tear_down(&path);
        assert_eq!(items[&key], expected_item);
        Ok(())
    }

    #[test]
    fn it_fails_to_get_a_nonexistent_item() -> BinaryMerkleTreeResult<()> {
        let seed = [0x03u8; KEY_LEN];
        let path = generate_path(seed);
        #[cfg(not(any(feature = "serde")))]
        let key = [0xAAu8; KEY_LEN];
        #[cfg(feature = "serde")]
        let key = [0xAAu8; KEY_LEN].into();
        let value = vec![0xFFu8];

        let mut bmt = Tree::open(&path, 160)?;
        let root = bmt.insert(None, &mut [key], &[value])?;

        #[cfg(not(any(feature = "serde")))]
        let nonexistent_key = [0xAB; KEY_LEN];
        #[cfg(feature = "serde")]
        let nonexistent_key = [0xAB; KEY_LEN].into();
        let items = bmt.get(&root, &mut [nonexistent_key])?;
        tear_down(&path);
        assert_eq!(items[&nonexistent_key], None);
        Ok(())
    }

    #[test]
    fn it_gets_items_from_a_small_balanced_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x04u8; KEY_LEN];
        let path = generate_path(seed);
        let mut keys = Vec::with_capacity(8);
        let mut values = Vec::with_capacity(8);
        for i in 0..8 {
            keys.push([i << 5u8; KEY_LEN].into());
            values.push(vec![i; KEY_LEN]);
        }

        let mut bmt = Tree::open(&path, 3)?;
        let root_hash = bmt.insert(None, &mut keys, &values)?;

        let items = bmt.get(&root_hash, &mut keys)?;
        tear_down(&path);
        for (key, value) in keys.into_iter().zip(values.into_iter()) {
            assert_eq!(Some(value), items[&key])
        }
        Ok(())
    }

    #[test]
    fn it_gets_items_from_a_small_unbalanced_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x05u8; KEY_LEN];
        let path = generate_path(seed);
        let mut keys = Vec::with_capacity(7);
        let mut values: Vec<Vec<u8>> = Vec::with_capacity(7);
        for i in 0..7 {
            keys.push([i << 5u8; KEY_LEN].into());
            values.push(vec![i; KEY_LEN]);
        }
        let mut bmt = Tree::open(&path, 3)?;

        let root_hash = bmt.insert(None, &mut keys, &values)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        tear_down(&path);
        for (key, value) in keys.into_iter().zip(values.into_iter()) {
            assert_eq!(items[&key], Some(value))
        }
        Ok(())
    }

    #[test]
    fn it_gets_items_from_a_tree_of_16() -> BinaryMerkleTreeResult<()> {
        let seed = [0x06u8; KEY_LEN];
        let path = generate_path(seed);

        let num_leaves = 16;
        let mut keys = Vec::with_capacity(num_leaves);
        let mut values: Vec<Vec<u8>> = Vec::with_capacity(num_leaves);
        for i in 0..num_leaves {
            keys.push([i as u8; KEY_LEN].into());
            values.push(vec![i as u8; KEY_LEN]);
        }

        let mut bmt = Tree::open(&path, 8)?;
        let root_hash = bmt.insert(None, &mut keys, &values)?;

        let items = bmt.get(&root_hash, &mut keys)?;
        tear_down(&path);
        for (key, value) in keys.into_iter().zip(values.into_iter()) {
            assert_eq!(items[&key], Some(value))
        }
        Ok(())
    }

    #[test]
    fn it_gets_items_from_a_tree_of_24() -> BinaryMerkleTreeResult<()> {
        let seed = [0x06u8; KEY_LEN];
        let path = generate_path(seed);

        let num_leaves = 24;
        let mut keys = Vec::with_capacity(num_leaves);
        let mut values = Vec::with_capacity(num_leaves);
        for i in 0..num_leaves {
            keys.push([i as u8; KEY_LEN].into());
            values.push(vec![i as u8; KEY_LEN]);
        }

        let mut bmt = Tree::open(&path, 8)?;
        let root_hash = bmt.insert(None, &mut keys, &values)?;

        let items = bmt.get(&root_hash, &mut keys)?;
        tear_down(&path);
        for (key, value) in keys.into_iter().zip(values.into_iter()) {
            assert_eq!(items[&key], Some(value))
        }
        Ok(())
    }

    #[test]
    fn it_gets_items_from_a_tree_of_25() -> BinaryMerkleTreeResult<()> {
        let seed = [0x06u8; KEY_LEN];
        let path = generate_path(seed);

        let num_leaves = 25;
        let mut keys = Vec::with_capacity(num_leaves);
        let mut values: Vec<Vec<u8>> = Vec::with_capacity(num_leaves);
        for i in 0..num_leaves {
            keys.push([i as u8; KEY_LEN].into());
            values.push(vec![i as u8; KEY_LEN]);
        }

        let mut bmt = Tree::open(&path, 8)?;
        let root_hash = bmt.insert(None, &mut keys, &values)?;

        let items = bmt.get(&root_hash, &mut keys)?;
        tear_down(&path);
        for (key, value) in keys.into_iter().zip(values.into_iter()) {
            assert_eq!(items[&key], Some(value))
        }
        Ok(())
    }

    #[test]
    fn it_gets_items_from_a_medium_balanced_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x06u8; KEY_LEN];
        let path = generate_path(seed);

        let num_leaves = 256;
        let mut keys = Vec::with_capacity(num_leaves);
        let mut values: Vec<Vec<u8>> = Vec::with_capacity(num_leaves);
        for i in 0..num_leaves {
            keys.push([i as u8; KEY_LEN].into());
            values.push(vec![i as u8; KEY_LEN]);
        }

        let mut bmt = Tree::open(&path, 8)?;
        let root_hash = bmt.insert(None, &mut keys, &values)?;

        let items = bmt.get(&root_hash, &mut keys)?;
        tear_down(&path);
        for (key, value) in keys.into_iter().zip(values.into_iter()) {
            assert_eq!(items[&key], Some(value))
        }
        Ok(())
    }

    #[test]
    fn it_gets_items_from_a_medium_unbalanced_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x07u8; KEY_LEN];
        let path = generate_path(seed);
        let num_leaves = 255;
        let mut keys = Vec::with_capacity(num_leaves);
        let mut values: Vec<Vec<u8>> = Vec::with_capacity(num_leaves);
        for i in 0..num_leaves {
            keys.push([i as u8; KEY_LEN].into());
            values.push(vec![i as u8; KEY_LEN]);
        }

        let mut bmt = Tree::open(&path, 8)?;
        let root_hash = bmt.insert(None, &mut keys, &values)?;

        let items = bmt.get(&root_hash, &mut keys)?;
        tear_down(&path);
        for (key, value) in keys.into_iter().zip(values.into_iter()) {
            assert_eq!(items[&key], Some(value))
        }
        Ok(())
    }

    #[test]
    fn it_gets_items_from_a_large_balanced_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x08u8; KEY_LEN];
        let path = generate_path(seed);

        #[cfg(not(any(feature = "groestl")))]
        let num_leaves = 8196;
        #[cfg(feature = "groestl")]
        let num_leaves = 1024;

        let mut keys: Vec<Array<KEY_LEN>> = Vec::with_capacity(num_leaves);
        let mut values: Vec<Vec<u8>> = Vec::with_capacity(num_leaves);
        for i in 0..num_leaves {
            let mut key = [0u8; KEY_LEN];
            key[0] = (i >> 8) as u8;
            key[1] = (i & 0xFF) as u8;
            values.push(key.to_vec());
            keys.push(key.into());
        }

        let mut bmt = Tree::open(&path, 16)?;
        let root_hash = bmt.insert(None, &mut keys, &values)?;

        let items = bmt.get(&root_hash, &mut keys)?;
        tear_down(&path);
        for (key, value) in keys.into_iter().zip(values.into_iter()) {
            assert_eq!(items[&key], Some(value))
        }
        Ok(())
    }

    #[test]
    fn it_gets_items_from_a_large_unbalanced_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x09u8; KEY_LEN];
        let path = generate_path(seed);

        #[cfg(not(any(feature = "groestl")))]
        let num_leaves = 8195;
        #[cfg(feature = "groestl")]
        let num_leaves = 1023;
        let mut keys = Vec::with_capacity(num_leaves);
        let mut values: Vec<Vec<u8>> = Vec::with_capacity(num_leaves);
        for i in 0..num_leaves {
            let mut key = [0u8; KEY_LEN];
            key[0] = (i >> 8) as u8;
            key[1] = (i & 0xFF) as u8;
            values.push(key.to_vec());
            keys.push(key.into());
        }

        let mut bmt = Tree::open(&path, 16)?;
        let root_hash = bmt.insert(None, &mut keys, &values)?;

        let items = bmt.get(&root_hash, &mut keys)?;
        tear_down(&path);
        for (key, value) in keys.into_iter().zip(values.into_iter()) {
            assert_eq!(items[&key], Some(value))
        }
        Ok(())
    }

    #[test]
    fn it_gets_items_from_a_complex_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x10u8; KEY_LEN];
        let path = generate_path(seed);

        // Tree description
        // Node (Letter)
        // Key (Number)
        // Value (Number)
        //
        // A     B      C      D     E     F     G     H     I     J     K     L     M     N     O     P
        // 0x00  0x40, 0x41, 0x60, 0x68, 0x70, 0x71, 0x72, 0x80, 0xC0, 0xC1, 0xE0, 0xE1, 0xE2, 0xF0, 0xF8
        // None, None, None, 0x01, 0x02, None, None, None, 0x03, None, None, None, None, None, 0x04, None
        let pop_key_d = [0x60u8; KEY_LEN].into(); // 0110_0000   96 (Dec)
        let pop_key_e = [0x68u8; KEY_LEN].into(); // 0110_1000  104 (Dec)
        let pop_key_i = [0x80u8; KEY_LEN].into(); // 1000_0000  128 (Dec)
        let pop_key_o = [0xF0u8; KEY_LEN].into(); // 1111_0000  240 (Dec)

        let mut populated_keys = [pop_key_d, pop_key_e, pop_key_i, pop_key_o];

        let pop_value_d = vec![0x01u8];
        let pop_value_e = vec![0x02u8];
        let pop_value_i = vec![0x03u8];
        let pop_value_o = vec![0x04u8];

        let populated_values = vec![
            pop_value_d.clone(),
            pop_value_e.clone(),
            pop_value_i.clone(),
            pop_value_o.clone(),
        ];

        let mut bmt = Tree::open(&path, 5)?;
        let root_node = bmt.insert(None, &mut populated_keys, &populated_values)?;

        let key_a = [0x00u8; KEY_LEN].into(); // 0000_0000     0 (Dec)
        let key_b = [0x40u8; KEY_LEN].into(); // 0100_0000    64 (Dec)
        let key_c = [0x41u8; KEY_LEN].into(); // 0100_0001    65 (Dec)
        let key_f = [0x70u8; KEY_LEN].into(); // 0111_0000   112 (Dec)
        let key_g = [0x71u8; KEY_LEN].into(); // 0111_0001   113 (Dec)
        let key_h = [0x72u8; KEY_LEN].into(); // 0111_0010   114 (Dec)
        let key_j = [0xC0u8; KEY_LEN].into(); // 1100_0000   192 (Dec)
        let key_k = [0xC1u8; KEY_LEN].into(); // 1100_0001   193 (Dec)
        let key_l = [0xE0u8; KEY_LEN].into(); // 1110_0000   224 (Dec)
        let key_m = [0xE1u8; KEY_LEN].into(); // 1110_0001   225 (Dec)
        let key_n = [0xE2u8; KEY_LEN].into(); // 1110_0010   226 (Dec)
        let key_p = [0xF8u8; KEY_LEN].into(); // 1111_1000   248 (Dec)

        let mut keys = vec![
            key_a, key_b, key_c, pop_key_d, pop_key_e, key_f, key_g, key_h, pop_key_i, key_j,
            key_k, key_l, key_m, key_n, pop_key_o, key_p,
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
        tear_down(&path);
        for (key, value) in keys.into_iter().zip(expected_values.into_iter()) {
            assert_eq!(items[&key], value);
        }
        Ok(())
    }

    #[test]
    fn it_returns_the_same_number_of_values_as_keys() -> BinaryMerkleTreeResult<()> {
        let seed = [0x11u8; KEY_LEN];
        let path = generate_path(seed);

        let initial_key = [0x00u8; KEY_LEN].into();
        let initial_value = vec![0xFFu8];

        let mut keys = Vec::with_capacity(256);
        for i in 0..256 {
            keys.push([i as u8; KEY_LEN].into());
        }

        let mut bmt = Tree::open(&path, 3)?;
        let root_node = bmt.insert(None, &mut [initial_key], &vec![initial_value.clone()])?;

        let items = bmt.get(&root_node, &mut keys)?;
        tear_down(&path);
        let first_value = Some(initial_value);
        for key in keys.into_iter() {
            if key == initial_key {
                assert_eq!(items[&key], first_value);
            } else {
                assert_eq!(items[&key], None);
            }
        }
        assert_eq!(items.len(), 256);
        Ok(())
    }

    #[test]
    fn it_inserts_two_leaf_nodes_into_empty_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x12u8; KEY_LEN];
        let path = generate_path(seed);

        let mut keys = vec![[0x00u8; KEY_LEN].into(), [0x01u8; KEY_LEN].into()];
        let values = vec![vec![0x02u8], vec![0x03u8]];

        let mut bmt = Tree::open(&path, 3)?;
        let root_hash = bmt.insert(None, &mut keys, &values)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        tear_down(&path);
        for (key, value) in keys.into_iter().zip(values.into_iter()) {
            assert_eq!(items[&key], Some(value))
        }
        Ok(())
    }

    #[test]
    fn it_inserts_two_leaf_nodes_into_empty_tree_with_first_bit_split() -> BinaryMerkleTreeResult<()>
    {
        let seed = [0x13u8; KEY_LEN];
        let path = generate_path(seed);

        let mut keys = vec![[0x00u8; KEY_LEN].into(), [0x80u8; KEY_LEN].into()];
        let values = vec![vec![0x02u8], vec![0x03u8]];

        let mut bmt = Tree::open(&path, 3)?;
        let root_hash = bmt.insert(None, &mut keys, &values)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        tear_down(&path);
        for (key, value) in keys.into_iter().zip(values.into_iter()) {
            assert_eq!(items[&key], Some(value))
        }
        Ok(())
    }

    #[test]
    fn it_inserts_a_leaf_node_into_empty_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x14u8; KEY_LEN];
        let path = generate_path(seed);

        let key = [0xAAu8; KEY_LEN].into();
        let data = vec![0xBBu8];

        let mut bmt = Tree::open(&path, 3)?;
        let new_root_hash = bmt.insert(None, &mut [key], &vec![data.clone()])?;
        let items = bmt.get(&new_root_hash, &mut vec![key])?;
        tear_down(&path);
        assert_eq!(items[&key], Some(data));
        Ok(())
    }

    #[test]
    fn it_inserts_multiple_leaf_nodes_into_empty_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x15u8; KEY_LEN];
        let path = generate_path(seed);

        let mut keys = vec![
            [0xAAu8; KEY_LEN].into(), // 1010_1010
            [0xBBu8; KEY_LEN].into(), // 1011_1011
            [0xCCu8; KEY_LEN].into(),
        ]; // 1100_1100
        let values = vec![vec![0xDDu8], vec![0xEEu8], vec![0xFFu8]];

        let mut bmt = Tree::open(&path, 3)?;
        let root_hash = bmt.insert(None, &mut keys, &values)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        tear_down(&path);
        for (key, value) in keys.into_iter().zip(values.into_iter()) {
            assert_eq!(items[&key], Some(value))
        }
        Ok(())
    }

    #[test]
    fn it_inserts_a_small_even_amount_of_nodes_into_empty_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x016u8; KEY_LEN];
        let path = generate_path(seed);

        let seed = [0xAAu8; KEY_LEN];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        let (mut keys, values) = prepare_inserts(KEY_LEN, &mut rng);

        let mut bmt = Tree::open(&path, 16)?;
        let root_hash = bmt.insert(None, &mut keys, &values)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        tear_down(&path);
        for (key, value) in keys.into_iter().zip(values.into_iter()) {
            assert_eq!(items[&key], Some(value))
        }
        Ok(())
    }

    #[test]
    fn it_inserts_a_small_odd_amount_of_nodes_into_empty_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x17u8; KEY_LEN];
        let path = generate_path(seed);

        let seed = [0xBBu8; KEY_LEN];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        let (mut keys, values) = prepare_inserts(31, &mut rng);

        let mut bmt = Tree::open(&path, 16)?;
        let root_hash = bmt.insert(None, &mut keys, &values)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        tear_down(&path);
        for (key, value) in keys.into_iter().zip(values.into_iter()) {
            assert_eq!(items[&key], Some(value))
        }
        Ok(())
    }

    #[test]
    fn it_inserts_a_medium_even_amount_of_nodes_into_empty_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x18u8; KEY_LEN];
        let path = generate_path(seed);

        let seed = [0xBBu8; KEY_LEN];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        let (mut keys, values) = prepare_inserts(256, &mut rng);

        let mut bmt = Tree::open(&path, 16)?;
        let root_hash = bmt.insert(None, &mut keys, &values)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        tear_down(&path);
        for (key, value) in keys.into_iter().zip(values.into_iter()) {
            assert_eq!(items[&key], Some(value))
        }
        Ok(())
    }

    #[test]
    fn it_inserts_a_medium_odd_amount_of_nodes_into_empty_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x19u8; KEY_LEN];
        let path = generate_path(seed);

        let seed = [0xBBu8; KEY_LEN];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        let (mut keys, values) = prepare_inserts(255, &mut rng);

        let mut bmt = Tree::open(&path, 16)?;
        let root_hash = bmt.insert(None, &mut keys, &values)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        tear_down(&path);
        for (key, value) in keys.into_iter().zip(values.into_iter()) {
            assert_eq!(items[&key], Some(value))
        }
        Ok(())
    }

    #[test]
    fn it_inserts_a_large_even_amount_of_nodes_into_empty_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x20u8; KEY_LEN];
        let path = generate_path(seed);

        let seed = [0xBBu8; KEY_LEN];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        #[cfg(not(any(feature = "groestl")))]
        let (mut keys, values) = prepare_inserts(4096, &mut rng);
        #[cfg(feature = "groestl")]
        let (mut keys, values) = prepare_inserts(256, &mut rng);

        let mut bmt = Tree::open(&path, 18)?;
        let root_hash = bmt.insert(None, &mut keys, &values)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        tear_down(&path);
        for (key, value) in keys.into_iter().zip(values.into_iter()) {
            assert_eq!(items[&key], Some(value))
        }
        Ok(())
    }

    #[test]
    fn it_inserts_a_large_odd_amount_of_nodes_into_empty_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x21u8; KEY_LEN];
        let path = generate_path(seed);

        let seed = [0xBBu8; KEY_LEN];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        #[cfg(not(any(feature = "groestl")))]
        let (mut keys, values) = prepare_inserts(4095, &mut rng);
        #[cfg(feature = "groestl")]
        let (mut keys, values) = prepare_inserts(256, &mut rng);

        let mut bmt = Tree::open(&path, 18)?;
        let root_hash = bmt.insert(None, &mut keys, &values)?;
        let items = bmt.get(&root_hash, &mut keys)?;
        tear_down(&path);
        for (key, value) in keys.into_iter().zip(values.into_iter()) {
            assert_eq!(items[&key], Some(value))
        }
        Ok(())
    }

    #[test]
    fn it_inserts_a_leaf_node_into_a_tree_with_one_item() -> BinaryMerkleTreeResult<()> {
        let seed = [0x22u8; KEY_LEN];
        let path = generate_path(seed);

        let first_key = [0xAAu8; KEY_LEN].into();
        let first_data = vec![0xBBu8];

        let second_key = [0xCCu8; KEY_LEN].into();
        let second_data = vec![0xDDu8];

        let mut bmt = Tree::open(&path, 3)?;
        let new_root_hash = bmt.insert(None, &mut [first_key], &[first_data.clone()])?;
        let second_root_hash = bmt.insert(
            Some(&new_root_hash),
            &mut [second_key],
            &[second_data.clone()],
        )?;

        let items = bmt.get(&second_root_hash, &mut [first_key, second_key])?;
        tear_down(&path);
        assert_eq!(items[&first_key], Some(first_data));
        assert_eq!(items[&second_key], Some(second_data));
        Ok(())
    }

    #[test]
    fn it_inserts_multiple_leaf_nodes_into_a_small_tree_with_existing_items(
    ) -> BinaryMerkleTreeResult<()> {
        let db_seed = [0x23u8; KEY_LEN];
        let path = generate_path(db_seed);

        let seed = [0xC7; KEY_LEN];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        let num_inserts = 2;
        let (mut initial_keys, initial_values) = prepare_inserts(num_inserts, &mut rng);

        let mut bmt = Tree::open(&path, 160)?;
        let first_root_hash = bmt.insert(None, &mut initial_keys, &initial_values)?;

        let (mut added_keys, added_values) = prepare_inserts(num_inserts, &mut rng);

        let second_root_hash =
            bmt.insert(Some(&first_root_hash), &mut added_keys, &added_values)?;

        let first_items = bmt.get(&first_root_hash, &mut initial_keys)?;
        let second_items = bmt.get(&second_root_hash, &mut added_keys)?;

        tear_down(&path);
        for (key, value) in initial_keys.into_iter().zip(initial_values.into_iter()) {
            assert_eq!(first_items[&key], Some(value));
        }
        for (key, value) in added_keys.into_iter().zip(added_values.into_iter()) {
            assert_eq!(second_items[&key], Some(value));
        }
        Ok(())
    }

    #[test]
    fn it_inserts_multiple_leaf_nodes_into_a_tree_with_existing_items() -> BinaryMerkleTreeResult<()>
    {
        let db_seed = [0x24u8; KEY_LEN];
        let path = generate_path(db_seed);

        let seed = [0xCAu8; KEY_LEN];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        #[cfg(not(any(feature = "groestl")))]
        let num_inserts = 4096;
        #[cfg(feature = "groestl")]
        let num_inserts = 256;
        let (mut initial_keys, initial_values) = prepare_inserts(num_inserts, &mut rng);

        let mut bmt = Tree::open(&path, 160)?;
        let first_root_hash = bmt.insert(None, &mut initial_keys, &initial_values)?;

        let (mut added_keys, added_values) = prepare_inserts(num_inserts, &mut rng);

        let second_root_hash =
            bmt.insert(Some(&first_root_hash), &mut added_keys, &added_values)?;

        let first_items = bmt.get(&first_root_hash, &mut initial_keys)?;
        let second_items = bmt.get(&second_root_hash, &mut added_keys)?;

        tear_down(&path);
        for (key, value) in initial_keys.into_iter().zip(initial_values.into_iter()) {
            assert_eq!(first_items[&key], Some(value));
        }
        for (key, value) in added_keys.into_iter().zip(added_values.into_iter()) {
            assert_eq!(second_items[&key], Some(value));
        }
        Ok(())
    }

    #[test]
    fn it_updates_an_existing_entry() -> BinaryMerkleTreeResult<()> {
        let seed = [0x25u8; KEY_LEN];
        let path = generate_path(seed);

        let key = [0xAAu8; KEY_LEN].into();
        let first_value = vec![0xBBu8];
        let second_value = vec![0xCCu8];

        let mut bmt = Tree::open(&path, 3)?;
        let first_root_hash = bmt.insert(None, &mut [key], &vec![first_value.clone()])?;
        let second_root_hash = bmt.insert(
            Some(&first_root_hash),
            &mut [key],
            &vec![second_value.clone()],
        )?;

        let first_item = bmt.get(&first_root_hash, &mut [key])?;
        let second_item = bmt.get(&second_root_hash, &mut [key])?;

        tear_down(&path);
        assert_eq!(first_item[&key], Some(first_value));
        assert_eq!(second_item[&key], Some(second_value));
        Ok(())
    }

    #[test]
    fn it_updates_multiple_existing_entries() -> BinaryMerkleTreeResult<()> {
        let seed = [0x26u8; KEY_LEN];
        let path = generate_path(seed);

        let seed = [0xEEu8; KEY_LEN];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        #[cfg(not(any(feature = "groestl")))]
        let (mut initial_keys, initial_values) = prepare_inserts(4096, &mut rng);
        #[cfg(feature = "groestl")]
        let (mut initial_keys, initial_values) = prepare_inserts(256, &mut rng);

        let mut updated_values = vec![];
        for i in 0..initial_keys.len() {
            let num = vec![i as u8; KEY_LEN];
            updated_values.push(num);
        }

        let mut bmt = Tree::open(&path, 160)?;
        let first_root_hash = bmt.insert(None, &mut initial_keys, &initial_values)?;
        let second_root_hash =
            bmt.insert(Some(&first_root_hash), &mut initial_keys, &updated_values)?;

        let initial_items = bmt.get(&first_root_hash, &mut initial_keys)?;
        let updated_items = bmt.get(&second_root_hash, &mut initial_keys)?;

        tear_down(&path);
        for (key, value) in initial_keys.iter().zip(initial_values.into_iter()) {
            assert_eq!(initial_items[key], Some(value));
        }
        for (key, value) in initial_keys.into_iter().zip(updated_values.into_iter()) {
            assert_eq!(updated_items[&key], Some(value));
        }
        Ok(())
    }

    #[test]
    fn it_does_not_panic_when_removing_a_nonexistent_node() -> BinaryMerkleTreeResult<()> {
        let seed = [0x27u8; KEY_LEN];
        let path = generate_path(seed);

        let mut bmt = Tree::open(&path, 160)?;
        let missing_root_hash = [0x00u8; KEY_LEN].into();
        bmt.remove(&missing_root_hash)?;
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_removes_a_node() -> BinaryMerkleTreeResult<()> {
        let seed = [0x28u8; KEY_LEN];
        let path = generate_path(seed);

        let key = [0x00u8; KEY_LEN].into();
        let data = vec![0x01u8];

        let mut bmt = Tree::open(&path, 160)?;
        let root_hash = bmt.insert(None, &mut [key], &vec![data.clone()])?;

        let inserted_data = bmt.get(&root_hash, &mut [key])?;

        assert_eq!(inserted_data[&key], Some(data));

        bmt.remove(&root_hash)?;

        let retrieved_values = bmt.get(&root_hash, &mut [key])?;

        assert_eq!(retrieved_values[&key], None);
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_removes_an_entire_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x29u8; KEY_LEN];
        let path = generate_path(seed);

        let seed = [0xBBu8; KEY_LEN];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        #[cfg(not(any(feature = "groestl")))]
        let (mut keys, values) = prepare_inserts(4096, &mut rng);
        #[cfg(feature = "groestl")]
        let (mut keys, values) = prepare_inserts(256, &mut rng);

        let mut bmt = Tree::open(&path, 160)?;

        let root_hash = bmt.insert(None, &mut keys, &values)?;
        let inserted_items = bmt.get(&root_hash, &mut keys)?;

        for (key, value) in keys.iter().zip(values.into_iter()) {
            assert_eq!(inserted_items[key], Some(value));
        }

        bmt.remove(&root_hash)?;
        let removed_items = bmt.get(&root_hash, &mut keys)?;

        tear_down(&path);

        for key in keys.into_iter() {
            assert_eq!(removed_items[&key], None);
        }
        Ok(())
    }

    #[test]
    fn it_removes_an_old_root() -> BinaryMerkleTreeResult<()> {
        let seed = [0x30u8; KEY_LEN];
        let path = generate_path(seed);

        let first_key = [0x00u8; KEY_LEN].into();
        let first_data = vec![0x01u8];

        let mut bmt = Tree::open(&path, 160)?;
        let first_root_hash = bmt.insert(None, &mut [first_key], &[first_data.clone()])?;

        let second_key = [0x02u8; KEY_LEN].into();
        let second_data = vec![0x03u8];

        let second_root_hash = bmt.insert(
            Some(&first_root_hash),
            &mut vec![second_key],
            &vec![second_data.clone()],
        )?;
        bmt.remove(&first_root_hash)?;

        let retrieved_items = bmt.get(&second_root_hash, &mut vec![first_key, second_key])?;
        tear_down(&path);
        assert_eq!(retrieved_items[&first_key], Some(first_data));
        assert_eq!(retrieved_items[&second_key], Some(second_data));
        Ok(())
    }

    #[test]
    fn it_removes_a_small_old_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x31u8; KEY_LEN];
        let path = generate_path(seed);

        let first_key = [0x00u8; KEY_LEN].into();
        let second_key = [0x01u8; KEY_LEN].into();
        let third_key = [0x02u8; KEY_LEN].into();
        let fourth_key = [0x03u8; KEY_LEN].into();

        let first_data = vec![0x04u8];
        let second_data = vec![0x05u8];
        let third_data = vec![0x06u8];
        let fourth_data = vec![0x07u8];

        let mut first_keys = vec![first_key, second_key];
        let first_entries = vec![first_data, second_data];
        let mut bmt = Tree::open(&path, 160)?;
        let first_root_hash = bmt.insert(None, &mut first_keys, &first_entries)?;

        let mut second_keys = vec![third_key, fourth_key];
        let second_entries = vec![third_data, fourth_data];
        let second_root_hash =
            bmt.insert(Some(&first_root_hash), &mut second_keys, &second_entries)?;
        bmt.remove(&first_root_hash)?;

        let items = bmt.get(
            &second_root_hash,
            &mut vec![first_key, second_key, third_key, fourth_key],
        )?;
        tear_down(&path);
        for (key, value) in first_keys.iter().zip(first_entries.into_iter()) {
            if let Some(v) = &items[key] {
                assert_eq!(*v, value);
            } else {
                panic!("None value found");
            }
        }
        for (key, value) in second_keys.iter().zip(second_entries.into_iter()) {
            if let Some(v) = &items[key] {
                assert_eq!(*v, value);
            } else {
                panic!("None value found");
            }
        }
        Ok(())
    }

    #[test]
    fn it_removes_an_old_large_root() -> BinaryMerkleTreeResult<()> {
        let seed = [0x32u8; KEY_LEN];
        let path = generate_path(seed);

        let seed = [0xBAu8; KEY_LEN];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        let (mut initial_keys, initial_values) = prepare_inserts(16, &mut rng);

        let mut bmt = Tree::open(&path, 160)?;
        let first_root_hash = bmt.insert(None, &mut initial_keys, &initial_values)?;

        let (mut added_keys, added_values) = prepare_inserts(16, &mut rng);

        let second_root_hash =
            bmt.insert(Some(&first_root_hash), &mut added_keys, &added_values)?;

        bmt.remove(&first_root_hash)?;
        let initial_items = bmt.get(&second_root_hash, &mut initial_keys)?;
        let added_items = bmt.get(&second_root_hash, &mut added_keys)?;
        tear_down(&path);
        for (key, value) in initial_keys.into_iter().zip(initial_values.into_iter()) {
            assert_eq!(initial_items[&key], Some(value));
        }
        for (key, value) in added_keys.into_iter().zip(added_values.into_iter()) {
            assert_eq!(added_items[&key], Some(value));
        }
        Ok(())
    }

    #[test]
    fn it_iterates_over_multiple_inserts_correctly() -> BinaryMerkleTreeResult<()> {
        let seed = [0x33u8; KEY_LEN];
        let path = generate_path(seed);

        let seed = [0xEFu8; KEY_LEN];
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let mut bmt = Tree::open(&path, 160)?;

        #[cfg(not(any(feature = "groestl")))]
        iterate_inserts(8, 100, &mut rng, &mut bmt)?;
        #[cfg(feature = "groestl")]
        iterate_inserts(8, 10, &mut rng, &mut bmt)?;

        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_inserts_with_compressed_nodes_that_are_not_descendants() -> BinaryMerkleTreeResult<()> {
        let seed = [0x34u8; KEY_LEN];
        let path = generate_path(seed);

        let mut bmt = Tree::open(&path, 160)?;

        let mut keys = vec![
            [0x00u8; KEY_LEN].into(),
            [0x01u8; KEY_LEN].into(),
            [0x02u8; KEY_LEN].into(),
            [0x10u8; KEY_LEN].into(),
            [0x20u8; KEY_LEN].into(),
        ];
        let values = vec![
            vec![0x00u8],
            vec![0x01u8],
            vec![0x02u8],
            vec![0x03u8],
            vec![0x04u8],
        ];

        let first_root = bmt.insert(None, &mut keys[0..2], &values[0..2])?;
        let second_root = bmt.insert(Some(&first_root), &mut keys[2..], &values[2..])?;

        let items = bmt.get(&second_root, &mut keys)?;
        tear_down(&path);
        for (key, value) in keys.into_iter().zip(values.into_iter()) {
            assert_eq!(items[&key], Some(value));
        }
        Ok(())
    }

    #[test]
    fn it_inserts_with_compressed_nodes_that_are_descendants() -> BinaryMerkleTreeResult<()> {
        let seed = [0x35u8; KEY_LEN];
        let path = generate_path(seed);

        let mut bmt = Tree::open(&path, 160)?;

        let mut keys = vec![
            [0x10u8; KEY_LEN].into(),
            [0x11u8; KEY_LEN].into(),
            [0x00u8; KEY_LEN].into(),
            [0x01u8; KEY_LEN].into(),
            [0x02u8; KEY_LEN].into(),
        ];
        let values = vec![
            vec![0x00u8],
            vec![0x01u8],
            vec![0x02u8],
            vec![0x03u8],
            vec![0x04u8],
        ];

        let sorted_data = vec![
            vec![0x02u8],
            vec![0x03u8],
            vec![0x04u8],
            vec![0x00u8],
            vec![0x01u8],
        ];

        let first_root = bmt.insert(None, &mut keys[0..2], &values[0..2])?;
        let second_root = bmt.insert(Some(&first_root), &mut keys[2..], &values[2..])?;

        let items = bmt.get(&second_root, &mut keys)?;
        tear_down(&path);
        for (key, value) in keys.into_iter().zip(sorted_data.into_iter()) {
            assert_eq!(items[&key], Some(value));
        }
        Ok(())
    }

    #[test]
    fn it_correctly_iterates_removals() -> BinaryMerkleTreeResult<()> {
        let seed = [0x36u8; KEY_LEN];
        let path = generate_path(seed);

        let seed = [0xA8u8; KEY_LEN];
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let mut bmt = Tree::open(&path, 160)?;

        #[cfg(not(any(feature = "groestl")))]
        iterate_removals(8, 100, 1, &mut rng, &mut bmt)?;
        #[cfg(feature = "groestl")]
        iterate_removals(8, 10, 1, &mut rng, &mut bmt)?;
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_correctly_increments_a_leaf_reference_count() -> BinaryMerkleTreeResult<()> {
        let seed = [0x37u8; KEY_LEN];
        let path = generate_path(seed);

        let mut bmt = Tree::open(&path, 160)?;

        let key = [0x00u8; KEY_LEN].into();
        let data = vec![0x00u8];

        let first_root = bmt.insert(None, &mut [key], &vec![data.clone()])?;
        let second_root = bmt.insert(Some(&first_root), &mut [key], &vec![data.clone()])?;
        bmt.remove(&first_root)?;
        let item = bmt.get(&second_root, &mut [key])?;

        tear_down(&path);
        assert_eq!(item[&key], Some(data));
        Ok(())
    }

    #[test]
    fn it_generates_a_simple_inclusion_proof() -> BinaryMerkleTreeResult<()> {
        let seed = [0x42u8; KEY_LEN];
        let path = generate_path(seed);

        let mut bmt = Tree::open(&path, 160)?;

        let key = [0x00u8; KEY_LEN].into();
        let data = vec![0x00u8];

        let root = bmt.insert(None, &mut [key], &vec![data.clone()])?;

        let inclusion_proof = bmt.generate_inclusion_proof(&root, key)?;
        Tree::verify_inclusion_proof(&root, key, &data, &inclusion_proof)?;
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_fails_an_invalid_simple_proof() -> BinaryMerkleTreeResult<()> {
        let seed = [0x4Cu8; KEY_LEN];
        let path = generate_path(seed);

        let mut bmt = Tree::open(&path, 160)?;

        let key = [0x00u8; KEY_LEN].into();
        let data = vec![0x00u8];

        let root = bmt.insert(None, &mut [key], &vec![data.clone()])?;

        let inclusion_proof = bmt.generate_inclusion_proof(&root, key)?;
        match Tree::verify_inclusion_proof(&[01u8; KEY_LEN].into(), key, &data, &inclusion_proof) {
            Ok(_) => return Err(Exception::new("Failed to detect invalid proof")),
            _ => {}
        }
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_generates_a_medium_size_inclusion_proof() -> BinaryMerkleTreeResult<()> {
        let seed = [0xE8u8; KEY_LEN];
        let path = generate_path(seed);
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        let num_entries = 256;

        let (mut keys, values) = prepare_inserts(num_entries, &mut rng);

        let mut bmt = Tree::open(&path, 160)?;

        let root = bmt.insert(None, &mut keys, &values)?;

        for i in 0..num_entries {
            let inclusion_proof = bmt.generate_inclusion_proof(&root, keys[i])?;
            Tree::verify_inclusion_proof(&root, keys[i], &values[i], &inclusion_proof)?;
        }
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_generates_a_large_size_inclusion_proof() -> BinaryMerkleTreeResult<()> {
        let seed = [0xFCu8; KEY_LEN];
        let path = generate_path(seed);
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        #[cfg(not(feature = "groestl"))]
        let num_entries = 4096;
        #[cfg(feature = "groestl")]
        let num_entries = 512;

        let (mut keys, values) = prepare_inserts(num_entries, &mut rng);

        let mut bmt = Tree::open(&path, 160)?;

        let root = bmt.insert(None, &mut keys, &values)?;

        for i in 0..num_entries {
            let inclusion_proof = bmt.generate_inclusion_proof(&root, keys[i])?;
            Tree::verify_inclusion_proof(&root, keys[i], &values[i], &inclusion_proof)?;
        }
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_fails_a_large_size_invalid_proof() -> BinaryMerkleTreeResult<()> {
        let seed = [0x61u8; KEY_LEN];
        let path = generate_path(seed);
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        #[cfg(not(feature = "groestl"))]
        let num_entries = 4096;
        #[cfg(feature = "groestl")]
        let num_entries = 512;

        let (mut keys, values) = prepare_inserts(num_entries, &mut rng);

        let mut bmt = Tree::open(&path, 160)?;

        let root = bmt.insert(None, &mut keys, &values)?;

        for i in 0..num_entries {
            let inclusion_proof = bmt.generate_inclusion_proof(&root, keys[i])?;
            if let Ok(_) = Tree::verify_inclusion_proof(
                &[0x03; KEY_LEN].into(),
                keys[i],
                &values[i],
                &inclusion_proof,
            ) {
                return Err(Exception::new("Failed to detect an invalid proof"));
            }
        }
        tear_down(&path);
        Ok(())
    }

    #[test]
    fn it_gets_one_key_from_a_small_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0xE6u8; KEY_LEN];
        let path = generate_path(seed);

        let key = [0x96u8; KEY_LEN].into();
        let value = vec![0xB3u8];

        let mut bmt = Tree::open(&path, 3)?;
        let root = bmt.insert(None, &mut [key], &[value.clone()])?;

        let retrieved_value = bmt.get_one(&root, &key)?.unwrap();
        tear_down(&path);
        assert_eq!(retrieved_value, value);
        Ok(())
    }

    #[test]
    fn it_gets_one_key_from_a_large_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x61u8; KEY_LEN];
        let path = generate_path(seed);
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        #[cfg(not(feature = "groestl"))]
        let num_entries = 4096;
        #[cfg(feature = "groestl")]
        let num_entries = 512;

        let (mut keys, values) = prepare_inserts(num_entries, &mut rng);

        let mut bmt = Tree::open(&path, 160)?;

        let root = bmt.insert(None, &mut keys, &values)?;

        let test_key = keys[keys.len() / 2];
        let test_value = &values[values.len() / 2];

        let retrieved_value = bmt.get_one(&root, &test_key)?.unwrap();
        tear_down(&path);
        assert_eq!(retrieved_value, *test_value);
        Ok(())
    }

    #[test]
    fn it_inserts_one_into_an_empty_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x61u8; KEY_LEN];
        let path = generate_path(seed);

        let key = [0x78u8; KEY_LEN].into();
        let value = vec![0x2Bu8];

        let mut bmt = Tree::open(&path, 2)?;
        let root = bmt.insert_one(None, &key, &value)?;

        let retrieved_value = bmt.get_one(&root, &key)?.unwrap();
        tear_down(&path);
        assert_eq!(retrieved_value, value);
        Ok(())
    }

    #[test]
    fn it_inserts_one_into_a_large_tree() -> BinaryMerkleTreeResult<()> {
        let seed = [0x51u8; KEY_LEN];
        let path = generate_path(seed);
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        #[cfg(not(feature = "groestl"))]
        let num_entries = 4096;
        #[cfg(feature = "groestl")]
        let num_entries = 512;

        let (mut keys, values) = prepare_inserts(num_entries, &mut rng);

        let mut bmt = Tree::open(&path, 160)?;

        let root = bmt.insert(None, &mut keys, &values)?;

        let test_key = [0x00u8; KEY_LEN].into();
        let test_value = vec![0x00u8];

        let new_root = bmt.insert_one(Some(&root), &test_key, &test_value)?;

        let retrieved_value = bmt.get_one(&new_root, &test_key)?.unwrap();
        tear_down(&path);
        assert_eq!(retrieved_value, test_value);

        Ok(())
    }

    test_key_size!(it_handles_key_size_of_two, 2, [0x94u8; 32], 16, 16);
    test_key_size!(it_handles_key_size_of_three, 3, [0x95u8; 32], 32, 32);
    test_key_size!(it_handles_key_size_of_four, 4, [0x96u8; 32], 64, 64);
    test_key_size!(it_handles_key_size_of_five, 5, [0x97u8; 32], 128, 128);
    test_key_size!(it_handles_key_size_of_six, 6, [0x98u8; 32], 256, 256);
    test_key_size!(it_handles_key_size_of_seven, 7, [0x99u8; 32], 512, 512);
    test_key_size!(it_handles_key_size_of_eight, 8, [0x9Au8; 32], 1024, 512);
    test_key_size!(it_handles_key_size_of_nine, 9, [0x9Bu8; 32], 2048, 512);
    test_key_size!(it_handles_key_size_of_ten, 10, [0x9Cu8; 32], 4096, 512);
    test_key_size!(it_handles_key_size_of_eleven, 11, [0x9Du8; 32], 4096, 512);
    test_key_size!(it_handles_key_size_of_twelve, 12, [0x9Eu8; 32], 4096, 512);
    test_key_size!(it_handles_key_size_of_thirteen, 13, [0x9Fu8; 32], 4096, 512);
    test_key_size!(it_handles_key_size_of_fourteen, 14, [0xA0u8; 32], 4096, 512);
    test_key_size!(it_handles_key_size_of_fifteen, 15, [0xA1u8; 32], 4096, 512);
    test_key_size!(it_handles_key_size_of_sixteen, 16, [0xA2u8; 32], 4096, 512);
    test_key_size!(
        it_handles_key_size_of_seventeen,
        17,
        [0xA3u8; 32],
        4096,
        512
    );
    test_key_size!(it_handles_key_size_of_eighteen, 18, [0xA4u8; 32], 4096, 512);
    test_key_size!(it_handles_key_size_of_nineteen, 19, [0xA5u8; 32], 4096, 512);
    test_key_size!(it_handles_key_size_of_twenty, 20, [0xA6u8; 32], 4096, 512);
    test_key_size!(
        it_handles_key_size_of_twenty_one,
        21,
        [0xA7u8; 32],
        4096,
        512
    );
    test_key_size!(
        it_handles_key_size_of_twenty_two,
        22,
        [0xA8u8; 32],
        4096,
        512
    );
    test_key_size!(
        it_handles_key_size_of_twenty_three,
        23,
        [0xA9u8; 32],
        4096,
        512
    );
    test_key_size!(
        it_handles_key_size_of_twenty_four,
        24,
        [0xAAu8; 32],
        4096,
        512
    );
    test_key_size!(
        it_handles_key_size_of_twenty_five,
        25,
        [0xABu8; 32],
        4096,
        512
    );
    test_key_size!(
        it_handles_key_size_of_twenty_six,
        26,
        [0xACu8; 32],
        4096,
        512
    );
    test_key_size!(
        it_handles_key_size_of_twenty_seven,
        27,
        [0xADu8; 32],
        4096,
        512
    );
    test_key_size!(
        it_handles_key_size_of_twenty_eight,
        28,
        [0xAEu8; 32],
        4096,
        512
    );
    test_key_size!(
        it_handles_key_size_of_twenty_nine,
        29,
        [0xAFu8; 32],
        4096,
        512
    );
    test_key_size!(it_handles_key_size_of_thirty, 30, [0xB0u8; 32], 4096, 512);
    test_key_size!(
        it_handles_key_size_of_thirty_one,
        31,
        [0xB1u8; 32],
        4096,
        512
    );
    test_key_size!(
        it_handles_key_size_of_thirty_two,
        32,
        [0xB2u8; 32],
        4096,
        512
    );
    test_key_size!(
        it_handles_key_size_of_thirty_three,
        33,
        [0xB2u8; 32],
        4096,
        512
    );

    fn generate_path(seed: [u8; KEY_LEN]) -> PathBuf {
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let suffix = rng.gen_range(1000..100000);
        let path_string = format!("Test_DB_{}", suffix);
        PathBuf::from(path_string)
    }

    fn tear_down(_path: &PathBuf) {
        #[cfg(feature = "rocksdb")]
        use std::fs::remove_dir_all;

        #[cfg(feature = "rocksdb")]
        remove_dir_all(&_path).unwrap();
    }

    fn prepare_inserts(
        num_entries: usize,
        rng: &mut StdRng,
    ) -> (Vec<Array<KEY_LEN>>, Vec<Vec<u8>>) {
        let mut keys: Vec<Array<KEY_LEN>> = Vec::with_capacity(num_entries);
        let mut data = Vec::with_capacity(num_entries);
        for _ in 0..num_entries {
            let mut key_value = [0u8; KEY_LEN];
            rng.fill(&mut key_value);
            keys.push(key_value.into());

            let data_value = (0..KEY_LEN).map(|_| rng.gen()).collect();
            data.push(data_value);
        }

        keys.sort();

        (keys, data)
    }

    fn iterate_inserts(
        entries_per_insert: usize,
        iterations: usize,
        rng: &mut StdRng,
        bmt: &mut Tree,
    ) -> BinaryMerkleTreeResult<(
        Vec<Option<Array<KEY_LEN>>>,
        Vec<Vec<Array<KEY_LEN>>>,
        Vec<Vec<Vec<u8>>>,
    )> {
        let mut state_roots: Vec<Option<Array<KEY_LEN>>> = Vec::with_capacity(iterations);
        let mut key_groups = Vec::with_capacity(iterations);
        let mut data_groups = Vec::with_capacity(iterations);
        state_roots.push(None);

        for i in 0..iterations {
            let prepare = prepare_inserts(entries_per_insert, rng);
            let mut keys = prepare.0;
            let values = prepare.1;

            key_groups.push(keys.clone());
            data_groups.push(values.clone());

            let previous_state_root = &state_roots[i];
            let previous_root;
            match previous_state_root {
                Some(r) => previous_root = Some(r),
                None => previous_root = None,
            }

            let new_root = bmt.insert(previous_root, &mut keys, &values)?;
            state_roots.push(Some(new_root.clone()));

            let retrieved_items = bmt.get(&new_root, &mut keys)?;
            for (key, value) in keys.into_iter().zip(values.into_iter()) {
                if let Some(v) = &retrieved_items[&key] {
                    assert_eq!(*v, value);
                } else {
                    panic!("None value found");
                }
            }

            for j in 0..key_groups.len() {
                let items = bmt.get(&new_root, &mut key_groups[j])?;
                for (key, value) in key_groups[j].iter().zip(data_groups[j].iter()) {
                    if let Some(v) = &items[key] {
                        assert_eq!(*v, *value);
                    } else {
                        panic!("None value found");
                    }
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
        let mut key_groups = inserts.1;
        let data_groups = inserts.2;

        for i in 1..iterations {
            if i % removal_frequency == 0 {
                let root;
                if let Some(r) = state_roots[i] {
                    root = r;
                } else {
                    panic!("state_roots[{}] is None", i);
                }
                bmt.remove(&root)?;
                for j in 0..iterations {
                    let items = bmt.get(&root, &mut key_groups[i])?;
                    if j % removal_frequency == 0 {
                        for key in key_groups[i].iter() {
                            assert_eq!(items[key], None);
                        }
                    } else {
                        for (key, value) in key_groups[i].iter().zip(data_groups[i].iter()) {
                            if let Some(v) = &items[key] {
                                assert_eq!(*v, *value);
                            } else {
                                panic!("None value found")
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
