use std::{collections::HashMap, default};

pub trait Node {
    const LOG2_D: u64;
    const TOTAL_LOG2_D: u64;
    const CHILD_LOG2_D: u64 = Self::TOTAL_LOG2_D - Self::LOG2_D;
    const DIM: u64 = 1 << Self::LOG2_D;
    // Size of this node (i.e. length of data array)
    const SIZE: usize = 1 << (Self::LOG2_D * 3);
    // Total size of node, including child size
    const TOTAL_SIZE: u64 = 1 << (Self::TOTAL_LOG2_D * 3);

    /// Give the bit index for the child node that contains position `p`
    fn bit_index_from_coords(p: [u32; 3]) -> usize {
        // @TODO: test this better
        // Relative coordinates of point to nearest node
        let p = p.map(|c| c & ((1 << Self::TOTAL_LOG2_D) - 1));
        // Relative coordinates of child node
        let [x, y, z] = p.map(|c| c >> Self::CHILD_LOG2_D);
        // Actual bit_index
        (z | (y << Self::LOG2_D) | (x << ((Self::LOG2_D) << 1)))
            .try_into()
            .unwrap() // @HACK: do actual error handling
    }
}

#[derive(Debug)]
pub struct LeafNode<ValueType, const LOG2_D: u64>
where
    [(); ((1 << (LOG2_D * 3)) / 64) as usize]:,
    [(); (1 << (LOG2_D * 3)) as usize]:,
{
    pub data: [LeafData<ValueType>; (1 << (LOG2_D * 3)) as usize],
    pub value_mask: [u64; ((1 << (LOG2_D * 3)) / 64) as usize],
    pub flags: u64,
}

impl<ValueType, const LOG2_D: u64> Node for LeafNode<ValueType, LOG2_D>
where
    [(); ((1 << (LOG2_D * 3)) / 64) as usize]:,
    [(); (1 << (LOG2_D * 3)) as usize]:,
{
    const LOG2_D: u64 = LOG2_D;
    const TOTAL_LOG2_D: u64 = LOG2_D;
}

impl<ValueType, const LOG2_D: u64> LeafNode<ValueType, LOG2_D>
where
    [(); ((1 << (LOG2_D * 3)) / 64) as usize]:,
    [(); (1 << (LOG2_D * 3)) as usize]:,
{
    pub fn new() -> Self {
        let data: [LeafData<ValueType>; (1 << (LOG2_D * 3)) as usize] =
            std::array::from_fn(|i| LeafData::Offset(Self::SIZE - i));
        let value_mask: [u64; ((1 << (LOG2_D * 3)) / 64) as usize] =
            [0; ((1 << (LOG2_D * 3)) / 64) as usize];
        let flags = 0;

        Self {
            data,
            value_mask,
            flags,
        }
    }
}

#[derive(Debug)]
pub enum LeafData<ValueType> {
    Offset(usize),
    Value(ValueType),
}

#[derive(Debug)]
pub struct InternalNode<ValueType, ChildType, const LOG2_D: u64>
where
    [(); (1 << (LOG2_D * 3)) as usize]:,
    [(); ((1 << (LOG2_D * 3)) / 64) as usize]:,
    ChildType: Node,
{
    pub data: [InternalData<ValueType, ChildType>; (1 << (LOG2_D * 3)) as usize],
    pub value_mask: [u64; ((1 << (LOG2_D * 3)) / 64) as usize],
    pub child_mask: [u64; ((1 << (LOG2_D * 3)) / 64) as usize],
    pub origin: [u32; 3],
}

impl<ValueType, ChildType, const LOG2_D: u64> InternalNode<ValueType, ChildType, LOG2_D>
where
    [(); ((1 << (LOG2_D * 3)) / 64) as usize]:,
    [(); (1 << (LOG2_D * 3)) as usize]:,
    ValueType: Default,
    ChildType: Node,
{
    pub fn new() -> Self {
        let data: [InternalData<ValueType, ChildType>; (1 << (LOG2_D * 3)) as usize] =
            std::array::from_fn(|_| InternalData::Tile(ValueType::default()));
        let value_mask: [u64; ((1 << (LOG2_D * 3)) / 64) as usize] =
            [0; ((1 << (LOG2_D * 3)) / 64) as usize];
        let child_mask: [u64; ((1 << (LOG2_D * 3)) / 64) as usize] =
            [0; ((1 << (LOG2_D * 3)) / 64) as usize];
        let origin = [0; 3];

        Self {
            data,
            value_mask,
            child_mask,
            origin,
        }
    }
}

impl<ValueType, ChildType, const LOG2_D: u64> Node for InternalNode<ValueType, ChildType, LOG2_D>
where
    [(); (1 << (LOG2_D * 3)) as usize]:,
    [(); ((1 << (LOG2_D * 3)) / 64) as usize]:,
    ChildType: Node,
{
    const LOG2_D: u64 = LOG2_D;
    const TOTAL_LOG2_D: u64 = LOG2_D + ChildType::TOTAL_LOG2_D;
}

#[derive(Debug)]
pub enum InternalData<ValueType, ChildType> {
    Node(Box<ChildType>),
    Tile(ValueType),
}

#[derive(Debug)]
pub struct RootNode<ValueType, ChildType: Node> {
    // @SPEED: Use a custom hash function
    pub map: HashMap<[u32; 3], RootData<ValueType, ChildType>>,
    pub background: ValueType,
}

#[derive(Debug)]
pub enum RootData<ValueType, ChildType> {
    Node(Box<ChildType>),
    Tile(ValueType, bool),
}

impl<ValueType, ChildType: Node> RootNode<ValueType, ChildType>
where
    ValueType: Default,
{
    fn new() -> Self {
        let map = HashMap::new();
        let background = ValueType::default();
        Self { map, background }
    }
}

impl<ValueType, ChildType: Node> RootNode<ValueType, ChildType> {
    pub fn root_key_from_coords(&self, p: [u32; 3]) -> [u32; 3] {
        p.map(|c| c & !((1 << ChildType::TOTAL_LOG2_D) - 1))
    }
}

#[derive(Debug)]
pub struct VDB<ValueType, ChildType: Node> {
    pub root: RootNode<ValueType, ChildType>,
    pub grid_descriptor: GridDescriptor,
}

#[derive(Debug, Clone)]
pub struct GridDescriptor {
    pub name: String,
    pub instance_parent: String,
    pub grid_type: String,
    pub block_pos: u64,
    pub end_pos: u64,
    pub compression: u32,
    pub meta_data: String,
}

impl<ValueType, ChildType: Node> VDB<ValueType, ChildType>
where
    ValueType: Default,
{
    pub fn new() -> Self {
        let root = <RootNode<ValueType, ChildType>>::new();
        let grid_descriptor = GridDescriptor {
            name: "Demo".to_string(),
            instance_parent: "What".to_string(),
            grid_type: "Whatt".to_string(),
            block_pos: 0,
            end_pos: 0,
            compression: 1,
            meta_data: "Whatt".to_string(),
        };

        Self {
            root,
            grid_descriptor,
        }
    }
}

type N3 = LeafNode<u64, 3>;
type N4 = InternalNode<u64, N3, 4>;
type N5 = InternalNode<u64, N4, 5>;
type Root345 = RootNode<u64, N5>;
type VDB345 = VDB<u64, N5>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn n345_mask_size_test() {
        let n345_masks: (usize, usize, usize) = {
            fn size<T>(_: *const T) -> usize {
                std::mem::size_of::<T>()
            }

            let null: *const N3 = std::ptr::null();
            let n3_mask_size = size(unsafe { &raw const (*null).value_mask }) / 8;

            let null: *const N4 = std::ptr::null();
            let n4_mask_size = size(unsafe { &raw const (*null).value_mask }) / 8;

            let null: *const N5 = std::ptr::null();
            let n5_mask_size = size(unsafe { &raw const (*null).value_mask }) / 8;

            (n3_mask_size, n4_mask_size, n5_mask_size)
        };

        assert_eq!(n345_masks, (8, 64, 512));
    }

    #[test]
    fn total_log2_d_test() {
        assert_eq!(8, 1 << N3::TOTAL_LOG2_D);
        assert_eq!(128, 1 << N4::TOTAL_LOG2_D);
        assert_eq!(4096, 1 << N5::TOTAL_LOG2_D);
    }

    #[test]
    fn bit_index_test() {
        assert_eq!(83, N3::bit_index_from_coords([1, 2, 3]));
        assert_eq!(3382, N4::bit_index_from_coords([121321, 212123, 3121]));
        assert_eq!(0, N5::bit_index_from_coords([1, 2, 3]));
    }
}
