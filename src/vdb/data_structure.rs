use std::collections::HashMap;

pub trait Node {
    const LOG2_D: u64;
    const DIM: u64 = 1 << Self::LOG2_D;
    const SIZE: u64 = 1 << (Self::LOG2_D * 3);
}

#[derive(Debug)]
pub struct LeafNode<ValueType, const LOG2_D: u64>
where
    [(); ((1 << (LOG2_D * 3)) / 64) as usize]:,
{
    data: LeafData<ValueType>,
    value_mask: [u64; ((1 << (LOG2_D * 3)) / 64) as usize],
    flags: u64,
}

impl<ValueType, const LOG2_D: u64> Node for LeafNode<ValueType, LOG2_D>
where
    [(); ((1 << (LOG2_D * 3)) / 64) as usize]:,
{
    const LOG2_D: u64 = LOG2_D;
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
    data: [InternalData<'a, ValueType, ChildType>; 1 << (LOG2_D * 3) as usize],
    value_mask: [u64; ((1 << (LOG2_D * 3)) / 64) as usize],
    child_mask: [u64; ((1 << (LOG2_D * 3)) / 64) as usize],
}

impl<'a, ValueType, ChildType, const LOG2_D: u64> Node
    for InternalNode<'a, ValueType, ChildType, LOG2_D>
where
    [(); 1 << (LOG2_D * 3) as usize]:,
    [(); ((1 << (LOG2_D * 3)) / 64) as usize]:,
    ChildType: Node,
{
    const LOG2_D: u64 = LOG2_D + ChildType::LOG2_D;
}

#[derive(Debug)]
pub enum InternalData<'a, ValueType, ChildType> {
    Child(&'a ChildType),
    Value(ValueType),
}

pub struct RootNode<'a, ValueType, ChildType: Node> {
    root_map: HashMap<u32, RootData<'a, ValueType, ChildType>>,
    background: ValueType,
}

pub enum RootData<'a, ValueType, ChildType> {
    Node(Option<&'a ChildType>),
    Tile(ValueType, bool),
}

type N0 = LeafNode<u64, 3>;
type N1<'a> = InternalNode<'a, u64, N0, 4>;
type N2<'a> = InternalNode<'a, u64, N1<'a>, 5>;
type RootType<'a> = RootNode<'a, u64, N2<'a>>;

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test() {
        let value_mask_size: usize = {
            fn size<T>(_: *const T) -> usize {
                std::mem::size_of::<T>()
            }

            let null: *const N2 = std::ptr::null();
            size(unsafe { &raw const (*null).value_mask })
        };

        println!("{} bits in mask", value_mask_size / 64 * 8);
        println!("{}", N0::SIZE);
        println!("{}", N1::SIZE);
        println!("{}", N2::SIZE);
        assert!(false);
    }
}
