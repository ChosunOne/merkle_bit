use std::path::PathBuf;
use std::error::Error;

use common::{Encode, Exception, Decode};
use common::traits::{Branch, Data, Hasher, IdentifyNode, Leaf};

use rocksdb::{DB, Options};

pub type BinaryMerkleTreeResult<T> = Result<T, Box<Error>>;

pub enum NodeVariant<BranchType, LeafType, DataType>
    where BranchType: Branch,
          LeafType: Leaf,
          DataType: Data {
    Branch(BranchType),
    Leaf(LeafType),
    Data(DataType)
}

enum TreeBranch {
    Zero,
    One
}

struct SplitPairs<'a, LeafType: 'a>
    where LeafType: Leaf {
    zeros: &'a [LeafType],
    ones: &'a [LeafType]
}

impl<'a, LeafType> SplitPairs<'a, LeafType>
    where LeafType: Leaf {
    pub fn new(zeros: &'a [LeafType], ones: &'a [LeafType]) -> SplitPairs<'a, LeafType>
        where LeafType: Leaf {
        SplitPairs {
            zeros: &zeros,
            ones: &ones
        }
    }
}

fn choose_branch(key: &[u8], bit: usize) -> TreeBranch {
    let index = bit / 8;
    let shift = bit % 8;
    let extracted_bit = (key[index] >> (7 - shift)) & 1;
    if extracted_bit == 0 {
        return TreeBranch::Zero
    } else {
        return TreeBranch::One
    }
}

fn split_pairs<LeafType>(sorted_pairs: &[LeafType], bit: usize) ->  SplitPairs<LeafType>
    where LeafType: Leaf {

    if let TreeBranch::Zero = choose_branch(sorted_pairs[sorted_pairs.len() - 1].get_key(), bit) {
        return SplitPairs::new(&sorted_pairs[0..sorted_pairs.len()], &sorted_pairs[0..0])
    }

    if let TreeBranch::One = choose_branch(sorted_pairs[0].get_key(), bit) {
        return SplitPairs::new(&sorted_pairs[0..0], &sorted_pairs[0..sorted_pairs.len()])
    }

    let mut min = 0;
    let mut max = sorted_pairs.len();
    let mut iterations = 0;
    while max - min > 1 {
        let bisect = (max - min) / 2 + min;
        match choose_branch(sorted_pairs[bisect].get_key(), bit) {
            TreeBranch::Zero => min = bisect,
            TreeBranch::One =>  max = bisect
        }
    }

    SplitPairs::new(&sorted_pairs[0..max], &sorted_pairs[max..sorted_pairs.len()])
}

pub struct BinaryMerkleTree {
    db: DB,
    depth: usize
}

impl BinaryMerkleTree {
    pub fn new(path: PathBuf, depth: usize) -> BinaryMerkleTreeResult<Self> {
        let db = DB::open_default(path)?;
        Ok(Self {
            db,
            depth
        })
    }

    pub fn get<BranchType, LeafType, DataType, HashResultType, NodeType>(&self, root_hash: &mut HashResultType, keys: &[&[u8]]) -> BinaryMerkleTreeResult<Vec<LeafType>>
        where BranchType: Branch,
              LeafType: Leaf,
              DataType: Data,
              HashResultType: AsRef<[u8]>,
              NodeType: IdentifyNode<BranchType, LeafType, DataType> + Encode + Decode {
        let retrieved_node = self.db.get(root_hash.as_ref())?;
        let encoded_node;
        match retrieved_node {
            Some(data) => encoded_node = data,
            None => return Err(Box::new(Exception::new("Failed to find root node in database")))
        }

        let mut nodes = Vec::with_capacity(10);
        let
        let mut node = NodeType::decode(&encoded_node.to_vec())?;



        Ok(vec![])
    }

    pub fn insert<LeafType, HashResultType>(&self, leaves: &mut Vec<LeafType>, root: Option<HashResultType>) -> BinaryMerkleTreeResult<()>
        where LeafType: Leaf + Ord + Eq,
              HashResultType: AsRef<[u8]> + Clone {
        leaves.sort();
        leaves.dedup_by(|a, b| a == b);

        let count_delta = 0;
        if let Some(root_hash) = root {
            if let Ok(data) = self.db.get(root_hash.as_ref()) {

            }
        }

        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use serialization::state::{MerkleNode as ProtoMerkleNode,
                               MerkleNode_oneof_node::branch as ProtoMerkleNodeBranch,
                               MerkleNode_oneof_node::data as ProtoMerkleNodeData,
                               MerkleNode_oneof_node::leaf as ProtoMerkleNodeLeaf,
                               Branch as ProtoBranch,
                               Leaf as ProtoLeaf,
                               Data as ProtoData};

    use blake2_rfc::blake2b::{Blake2b, Blake2bResult};
    use protobuf::Message as ProtoMessage;


    impl Hasher for Blake2b {
        type HashType = Blake2b;
        type HashResultType = Blake2bResult;

        fn new(size: usize) -> Self::HashType {
            Blake2b::new(size)
        }
        fn update(&mut self, data: &[u8]) {
            Blake2b::update(self, data)
        }
        fn finalize(self) -> Self::HashResultType {
            Blake2b::finalize(self)
        }
    }

    impl Branch for ProtoBranch {
        fn get_count(&self) -> u64 {
            ProtoBranch::get_count(self)
        }

        fn get_zero(&self) -> &[u8] {
            ProtoBranch::get_zero(self)
        }

        fn get_one(&self) -> &[u8] {
            ProtoBranch::get_one(self)
        }
    }

    impl Leaf for ProtoLeaf {
        fn get_key(&self) -> &[u8] {
            ProtoLeaf::get_key(self)
        }

        fn get_data(&self) -> &[u8] {
            ProtoLeaf::get_data(self)
        }
    }

    impl Data for ProtoData {
        fn get_value(&self) -> &[u8] {
            ProtoData::get_value(self)
        }
    }

    impl IdentifyNode<ProtoBranch, ProtoLeaf, ProtoData> for ProtoMerkleNode {
        fn get_variant(&self)
                       -> BinaryMerkleTreeResult<NodeVariant<ProtoBranch, ProtoLeaf, ProtoData>>
            where ProtoBranch: Branch,
                  ProtoLeaf: Leaf,
                  ProtoData: Data, {
            match self.node {
                Some(ref node_type) => {
                    match node_type {
                        ProtoMerkleNodeBranch(branch) => return Ok(NodeVariant::Branch(branch.clone())),
                        ProtoMerkleNodeData(data) => return Ok(NodeVariant::Data(data.clone())),
                        ProtoMerkleNodeLeaf(leaf) => return Ok(NodeVariant::Leaf(leaf.clone()))
                    }
                },
                None => return Err(Box::new(Exception::new("Failed to distinguish node type")))
            }
        }
    }

    impl Leaf for [u8; 1] {
        fn get_key(&self) -> &[u8] {
            self
        }

        fn get_data(&self) -> &[u8] {
            self
        }
    }

    #[test]
    fn it_recognizes_a_hasher() {
        let mut blake = Blake2b::new(32);
        let data = [0u8; 32];
        blake.update(&data);
        let hash = blake.finalize();
        let expected_hash = [
            137, 235,  13, 106, 138, 105, 29, 174,
             44, 209,  94, 208,  54, 153, 49, 206,
             10, 148, 158, 202, 250,  92, 63, 147,
            248,  18,  24,  51, 100, 110, 21, 195];
        assert_eq!(hash.as_bytes(), expected_hash);
    }

    #[test]
    fn it_splits_an_all_zeros_sorted_list_of_pairs() {
        let leaves = [
        [0x00], [0x00], [0x00], [0x00], [0x00],
        [0x00], [0x00], [0x00], [0x00], [0x00]];

        let result = split_pairs(&leaves, 0);
        assert_eq!(result.zeros.len(), 10);
        assert_eq!(result.ones.len(), 0);
        for i in 0..result.zeros.len() {
            assert_eq!(result.zeros[i], [0x00]);
        }
    }

    #[test]
    fn it_splits_an_all_ones_sorted_list_of_pairs() {
        let leaves = [
            [0xFF], [0xFF], [0xFF], [0xFF], [0xFF],
            [0xFF], [0xFF], [0xFF], [0xFF], [0xFF]];

        let result = split_pairs(&leaves, 0);
        assert_eq!(result.zeros.len(), 0);
        assert_eq!(result.ones.len(), 10);
        for i in 0..result.ones.len() {
            assert_eq!(result.ones[i], [0xFF]);
        }
    }

    #[test]
    fn it_splits_an_even_length_sorted_list_of_pairs() {
        let leaves = [
            [0x00], [0x00], [0x00], [0x00], [0x00],
            [0xFF], [0xFF], [0xFF], [0xFF], [0xFF]];

        let result = split_pairs(&leaves, 0);
        assert_eq!(result.zeros.len(), 5);
        assert_eq!(result.ones.len(), 5);
        for i in 0..result.zeros.len() {
            assert_eq!(result.zeros[i], [0x00]);
        }
        for i in 0..result.ones.len() {
            assert_eq!(result.ones[i], [0xFF]);
        }
    }

    #[test]
    fn it_splits_an_odd_length_sorted_list_of_pairs_with_more_zeros() {
        let leaves = [
            [0x00], [0x00], [0x00], [0x00], [0x00], [0x00],
            [0xFF], [0xFF], [0xFF], [0xFF], [0xFF]];

        let result = split_pairs(&leaves, 0);
        assert_eq!(result.zeros.len(), 6);
        assert_eq!(result.ones.len(), 5);
        for i in 0..result.zeros.len() {
            assert_eq!(result.zeros[i], [0x00]);
        }
        for i in 0..result.ones.len() {
            assert_eq!(result.ones[i], [0xFF]);
        }
    }

    #[test]
    fn it_splits_an_odd_length_sorted_list_of_pairs_with_more_ones() {
        let leaves = [
            [0x00], [0x00], [0x00], [0x00], [0x00],
            [0xFF], [0xFF], [0xFF], [0xFF], [0xFF], [0xFF]];

        let result = split_pairs(&leaves, 0);
        assert_eq!(result.zeros.len(), 5);
        assert_eq!(result.ones.len(), 6);
        for i in 0..result.zeros.len() {
            assert_eq!(result.zeros[i], [0x00]);
        }
        for i in 0..result.ones.len() {
            assert_eq!(result.ones[i], [0xFF]);
        }
    }

}