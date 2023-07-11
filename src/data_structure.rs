use bitvec::prelude::*;

#[derive(Debug)]
pub struct LeafNode<ValueType, const LOG2_D: u32> {
    leaf_data: LeafData<ValueType>,
    value_mask: BitVec<u64, Lsb0>,
    flags: u64,
}

impl<ValueType, const LOG2_D: u32> LeafNode<ValueType, LOG2_D> {
    const SIZE: u32 = 1 << (LOG2_D * 3);
}

#[derive(Debug)]
pub enum LeafData<ValueType> {
    Offset(u64),
    Value(ValueType),
}

pub struct InternalNode<'a, ValueType, ChildType, const LOG2_D: u32>
where
    [(); 1 << (LOG2_D * 3) as usize]:,
{
    internal_data: [InternalData<'a, ValueType, ChildType>; 1 << (LOG2_D * 3) as usize],
    value_mask: BitVec<u64, Lsb0>,
    child_mask: BitVec<u64, Lsb0>,
}

impl<'a, ValueType, ChildType, const LOG2_D: u32> InternalNode<'a, ValueType, ChildType, LOG2_D>
where
    [(); 1 << (LOG2_D * 3) as usize]:,
{
    const SIZE: u32 = 1 << (LOG2_D * 3);
}

#[derive(Debug)]
pub enum InternalData<'a, ValueType, ChildType> {
    Child(&'a ChildType),
    Value(ValueType),
}

type MyLeaf = LeafNode<u64, 1>;
type MyIntern<'a> = InternalNode<'a, u64, MyLeaf, 1>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let leaf: MyLeaf = LeafNode {
            leaf_data: LeafData::Value(2),
            value_mask: bitvec![u64, Lsb0; 0; 8],
            flags: 7,
        };
        let intern: MyIntern = InternalNode {
            internal_data: [
                InternalData::Value(2),
                InternalData::Child(&leaf),
                InternalData::Value(2),
                InternalData::Value(2),
                InternalData::Value(2),
                InternalData::Value(2),
                InternalData::Value(2),
                InternalData::Value(2),
            ],
            value_mask: bitvec![u64, Lsb0; 0; 8],
            child_mask: bitvec![u64, Lsb0; 0; 8],
        };
        println!("{:?}", MyLeaf::SIZE);
        println!("{:?}", MyIntern::SIZE);
        println!("{:?}", intern.internal_data[1]);
        assert!(false);
    }
}
