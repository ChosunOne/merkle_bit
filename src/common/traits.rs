use common::binary_merkle_tree::BinaryMerkleTreeResult;
use common::binary_merkle_tree::NodeVariant;

pub trait Hasher {
    type HashType;
    type HashResultType;
    fn new(size: usize) -> Self::HashType;
    fn update(&mut self, data: &[u8]);
    fn finalize(self) -> Self::HashResultType;
}

pub trait Branch {
    fn get_count(&self) -> u64;
    fn get_zero(&self) -> &[u8];
    fn get_one(&self) -> &[u8];
}

pub trait Leaf {
    fn get_key(&self) -> &[u8];
    fn get_data(&self) -> &[u8];
}

pub trait Data {
    fn get_value(&self) -> &[u8];
}

pub trait IdentifyNode<BranchType, LeafType, DataType>
    where BranchType: Branch,
          LeafType: Leaf,
          DataType: Data {
    fn get_variant(&self) -> BinaryMerkleTreeResult<NodeVariant<BranchType, LeafType, DataType>>;
}