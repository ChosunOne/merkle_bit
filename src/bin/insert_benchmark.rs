use std::error::Error;

use starling::constants::KEY_LEN;
use starling::hash_tree::HashTree;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

fn main() -> Result<(), Box<Error>> {
    let seed = [0xBBu8; KEY_LEN];
    let mut rng: StdRng = SeedableRng::from_seed(seed);

    let mut tree = HashTree::new(160)?;

    let iterations = 200;

    for _ in 0..iterations {
        let prepare = prepare_inserts(1000, &mut rng);
        let key_values = prepare.0;
        let mut keys = key_values.iter().collect::<Vec<_>>();
        let data_values = prepare.1;
        let mut data = data_values.iter().collect::<Vec<_>>();

        tree.insert(None, &mut keys, &mut data)?;
    }

    Ok(())
}

fn prepare_inserts(num_entries: usize, rng: &mut StdRng) -> (Vec<[u8; KEY_LEN]>, Vec<Vec<u8>>) {
    let mut keys = Vec::with_capacity(num_entries);
    let mut data = Vec::with_capacity(num_entries);
    for _ in 0..num_entries {
        let mut key_value = [0u8; KEY_LEN];
        rng.fill(&mut key_value);
        keys.push(key_value);

        let data_value = (0..KEY_LEN).map(|_| rng.gen()).collect();
        data.push(data_value);
    }

    keys.sort();

    (keys, data)
}
