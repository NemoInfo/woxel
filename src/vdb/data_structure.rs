use bitflags::bitflags;
use cgmath::{num_traits::ToPrimitive, Zero};
use std::{
    any::type_name,
    collections::HashMap,
    io::{Read, Seek, SeekFrom},
};

pub trait Node {
    const LOG2_D: u64;
    const LOG2_DD: u64 = Self::LOG2_D * 2;
    const TOTAL_LOG2_D: u64;
    const CHILD_TOTAL_LOG2_D: u64 = Self::TOTAL_LOG2_D - Self::LOG2_D;
    const DIM: u64 = 1 << Self::LOG2_D;
    // Size of this node (i.e. length of data array)
    const SIZE: usize = 1 << (Self::LOG2_D * 3);
    // Total size of node, including child size
    const TOTAL_SIZE: u64 = 1 << (Self::TOTAL_LOG2_D * 3);

    /// Give the bit index for the child node that contains position `p`
    fn bit_index_from_coords(p: [u32; 3]) -> usize {
        // @SPEED: Maybe don't use iterators
        // Relative coordinates of point to nearest node
        let p = p.map(|c| c & ((1 << Self::TOTAL_LOG2_D) - 1));
        // Relative coordinates of child node
        let [x, y, z] = p.map(|c| c >> Self::CHILD_TOTAL_LOG2_D);
        // Actual bit_index
        (z | (y << Self::LOG2_D) | (x << ((Self::LOG2_D) << 1))) as usize
    }

    // fn pretty_print_inner(&self, input: &mut String, offset: usize);
}

#[derive(Debug)]
pub struct LeafNode<ValueType, const LOG2_D: u64>
where
    [(); ((1 << (LOG2_D * 3)) / 64) as usize]:,
    [(); (1 << (LOG2_D * 3)) as usize]:,
    //    ValueType: std::fmt::Display,
{
    pub data: [LeafData<ValueType>; (1 << (LOG2_D * 3)) as usize],
    pub value_mask: [u64; ((1 << (LOG2_D * 3)) / 64) as usize],
    pub flags: u64,
}

impl<ValueType, const LOG2_D: u64> Node for LeafNode<ValueType, LOG2_D>
where
    [(); ((1 << (LOG2_D * 3)) / 64) as usize]:,
    [(); (1 << (LOG2_D * 3)) as usize]:,
    //   ValueType: std::fmt::Display,
{
    const LOG2_D: u64 = LOG2_D;
    const TOTAL_LOG2_D: u64 = LOG2_D;

    // fn pretty_print_inner(&self, input: &mut String, offset: usize) {
    //     let border: String = " ".repeat(offset);
    //     *input += &format!("{}{}\n", border, type_name::<Self>(),);
    //     *input += &format!("{}data:\n", border);
    //     *input += &border.clone();
    //     for val in &self.data {
    //         match val {
    //             LeafData::Offset(_) => {
    //                 *input += "o";
    //             }
    //             LeafData::Value(v) => {
    //                 *input += &format!("{}", v);
    //             }
    //         }
    //     }
    //     *input += "\n";
    // }
}

impl<ValueType, const LOG2_D: u64> LeafNode<ValueType, LOG2_D>
where
    [(); ((1 << (LOG2_D * 3)) / 64) as usize]:,
    [(); (1 << (LOG2_D * 3)) as usize]:,
    //    ValueType: std::fmt::Display,
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
    pub origin: [i32; 3],
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

    // fn pretty_print_inner(&self, input: &mut String, offset: usize) {
    //     let border: String = " ".repeat(offset);
    //     *input += &format!(
    //         "{}{} {} {}: {}\n",
    //         border,
    //         self.origin[0],
    //         self.origin[1],
    //         self.origin[2],
    //         type_name::<Self>(),
    //     );
    //     *input += &format!("{}data:\n", border);
    //     for val in &self.data {
    //         match val {
    //             InternalData::Tile(_) => {}
    //             InternalData::Node(child) => {
    //                 child.pretty_print_inner(input, offset + 2);
    //             }
    //         }
    //     }
    //     *input += "\n";
    // }
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

    // pub fn pretty_print(&self, input: &mut String) {
    //     for (key, val) in self.map.iter() {
    //         if let RootData::Node(node) = val {
    //             *input += &format!(
    //                 "{} {} {}: {}\n",
    //                 key[0],
    //                 key[1],
    //                 key[2],
    //                 type_name::<ChildType>()
    //             );
    //             node.pretty_print_inner(input, 2);
    //         }
    //     }
    // }
}

impl<ValueType, ChildType: Node> RootNode<ValueType, ChildType> {
    pub fn root_key_from_coords(p: [u32; 3]) -> [u32; 3] {
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
    /// If not empty, the name of another grid that shares this grid's tree
    pub instance_parent: String,
    pub grid_type: String,
    /// Location in the stream where the grid data is stored
    pub grid_pos: u64,
    /// Location in the stream where the grid blocks are stored
    pub block_pos: u64,
    /// Location in the stream where the next grid descriptor begins
    pub end_pos: u64,
    pub compression: Compression,
    pub meta_data: Metadata,
    pub bbox_min: cgmath::Vector3<i32>,
}

impl GridDescriptor {
    pub fn seek_to_grid<R: Read + Seek>(&self, reader: &mut R) -> Result<u64, std::io::Error> {
        reader.seek(SeekFrom::Start(self.grid_pos))
    }

    pub fn world_to_u(&self, p: [i32; 3]) -> [u32; 3] {
        p.map(|c| (c + 1000) as u32)
    }
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
            grid_pos: 0,
            block_pos: 0,
            end_pos: 0,
            compression: Compression::NONE,
            meta_data: Default::default(),
            bbox_min: cgmath::Vector3::zero(),
        };

        Self {
            root,
            grid_descriptor,
        }
    }
}

#[derive(Debug)]
pub enum VdbEndpoint<ValueType> {
    Offs(usize),
    Leaf(ValueType),
    Innr(ValueType, u8),
    Root(ValueType),
    Bkgr(ValueType),
}

#[derive(Debug, Default, Clone)]
pub struct Metadata(pub HashMap<String, MetadataValue>);

#[derive(Debug, Clone, PartialEq)]
pub enum MetadataValue {
    String(String),
    Vec3i(cgmath::Vector3<i32>),
    I32(i32),
    I64(i64),
    Float(f32),
    Bool(bool),
    Unknown { name: String, data: Vec<u8> },
}

bitflags! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Compression: u32 {
        const NONE = 0;
        const ZIP = 0x1;
        const ACTIVE_MASK = 0x2;
        const BLOSC = 0x4;
        const DEFAULT_COMPRESSION = Self::BLOSC.bits() | Self::ACTIVE_MASK.bits();
    }
}

#[derive(Debug, Default)]
pub struct ArchiveHeader {
    /// The version of the file that was read
    pub file_version: u32,
    /// The version of the library that was used to create the file that was read
    pub library_major: u32,
    pub library_minor: u32,
    /// Unique tag, a random 16-byte (128-bit) value, stored as a string format.
    pub uuid: String,
    /// Flag indicating whether the input stream contains grid offsets and therefore supports partial reading
    pub has_grid_offsets: bool,
    /// Flags indicating whether and how the data stream is compressed
    pub compression: Compression,
    /// the number of grids on the input stream
    pub grid_number: u32,
    /// The metadata for the input stream
    pub meta_data: Metadata,
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
