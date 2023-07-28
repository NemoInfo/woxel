use std::collections::HashMap;

pub trait Node {
    const LOG2_D: u64;
    const TOTAL_LOG2_D: u64;
    const CHILD_LOG2_D: u64 = Self::TOTAL_LOG2_D - Self::LOG2_D;
    const DIM: u64 = 1 << Self::LOG2_D;
    // Size of this node
    // @TODO: Is this usefull? It seems to not include value sizes for leaf nodes
    const SIZE: u64 = 1 << (Self::LOG2_D * 3);
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

#[derive(Debug)]
pub enum LeafData<ValueType> {
    Offset(u64),
    Value(ValueType),
}

#[derive(Debug)]
pub struct InternalNode<'a, ValueType, ChildType, const LOG2_D: u64>
where
    [(); 1 << (LOG2_D * 3) as usize]:,
    [(); ((1 << (LOG2_D * 3)) / 64) as usize]:,
    ChildType: Node,
{
    pub data: [InternalData<'a, ValueType, ChildType>; 1 << (LOG2_D * 3) as usize],
    pub value_mask: [u64; ((1 << (LOG2_D * 3)) / 64) as usize],
    pub child_mask: [u64; ((1 << (LOG2_D * 3)) / 64) as usize],
    pub origin: [u32; 3],
}

impl<'a, ValueType, ChildType, const LOG2_D: u64> Node
    for InternalNode<'a, ValueType, ChildType, LOG2_D>
where
    [(); 1 << (LOG2_D * 3) as usize]:,
    [(); ((1 << (LOG2_D * 3)) / 64) as usize]:,
    ChildType: Node,
{
    const LOG2_D: u64 = LOG2_D;
    const TOTAL_LOG2_D: u64 = LOG2_D + ChildType::TOTAL_LOG2_D;
}

#[derive(Debug)]
pub enum InternalData<'a, ValueType, ChildType> {
    Node(&'a ChildType),
    Tile(ValueType),
}

#[derive(Debug)]
pub struct RootNode<'a, ValueType, ChildType: Node> {
    // @SPEED: Use a custom hash function
    pub root_map: HashMap<[u32; 3], RootData<'a, ValueType, ChildType>>,
    pub background: ValueType,
}

#[derive(Debug)]
pub enum RootData<'a, ValueType, ChildType> {
    Node(&'a ChildType),
    Tile(ValueType, bool),
    Empty,
}

impl<'a, ValueType, ChildType: Node> RootNode<'a, ValueType, ChildType> {
    fn root_key_from_coords(&self, p: [u32; 3]) -> [u32; 3] {
        p.map(|c| c & !((1 << ChildType::TOTAL_LOG2_D) - 1))
    }
}

#[derive(Debug)]
pub struct VDB<'a, ValueType, ChildType: Node> {
    pub root: RootNode<'a, ValueType, ChildType>,
    pub grid_descriptor: GridDescriptor,
}

macro_rules! parse {
    ($t: tt) => {
        println!("{:?}", std::any::type_name::<N5>());
    };
}

impl<'a, ValueType, ChildType: Node> VDB<'a, ValueType, ChildType> {
    fn new(grid_descriptor: GridDescriptor) -> Self {
        todo!()
    }

    fn set_voxel(&mut self, p: [u32; 3], v: ValueType) {
        let root_key = self.root.root_key_from_coords(p);
        match self.root.root_map.get_mut(&root_key) {
            None => todo!(),
            Some(RootData::Node(node)) => {
                let bit_index = <ChildType as Node>::bit_index_from_coords(p);
            }
            Some(RootData::Tile(_, _)) => todo!(),
            Some(RootData::Empty) => todo!(),
        }
        todo!()
    }
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

pub type N3 = LeafNode<u64, 3>;
pub type N4<'a> = InternalNode<'a, u64, N3, 4>;
pub type N5<'a> = InternalNode<'a, u64, N4<'a>, 5>;
pub type Root345<'a> = RootNode<'a, u64, N5<'a>>;
pub type VDB345<'a> = VDB<'a, u64, N5<'a>>;

#[cfg(test)]
mod tests {
    use std::any::type_name;

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

        assert!(false);
    }
}
// InternalNode<u64, InternalNode<u64, LeafNode<u64, 3>, 4>, 5>
// woxel::vdb::data_structure::InternalNode<u64, woxel::vdb::data_structure::InternalNode<u64, woxel::vdb::data_structure::LeafNode<u64, 3>, 4>, 5>
