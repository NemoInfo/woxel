use crate::vdb::data_structure::*;

pub type N3<ValueType> = LeafNode<ValueType, 3>;
pub type N4<ValueType> = InternalNode<ValueType, N3<ValueType>, 4>;
pub type N5<ValueType> = InternalNode<ValueType, N4<ValueType>, 5>;
pub type Root345<ValueType> = RootNode<ValueType, N5<ValueType>>;
pub type VDB345<ValueType> = VDB<ValueType, N5<ValueType>>;

impl<'a, ValueType> VDB345<ValueType>
where
    ValueType: Default + std::fmt::Display,
{
    pub fn set_voxel(&mut self, p: [u32; 3], v: ValueType) {
        let root_key = self.root.root_key_from_coords(p);
        let bit_index_4 = <N5<ValueType>>::bit_index_from_coords(p);
        let bit_index_3 = <N4<ValueType>>::bit_index_from_coords(p);
        let bit_index_0 = <N3<ValueType>>::bit_index_from_coords(p);

        let root_entry = self
            .root
            .map
            .entry(root_key)
            .or_insert_with(|| RootData::Node(Box::new(<N5<ValueType>>::new())));

        if let RootData::Tile(..) = root_entry {
            *root_entry = RootData::Node(Box::new(<N5<ValueType>>::new()));
        }

        if let RootData::Node(node_5) = root_entry {
            let node_5_entry = &mut node_5.data[bit_index_4];
            if let InternalData::Tile(..) = node_5_entry {
                *node_5_entry = InternalData::Node(Box::new(<N4<ValueType>>::new()));
            }

            node_5.child_mask[bit_index_4 >> 6] |= 1 << (bit_index_4 & (64 - 1));
            if let InternalData::Node(node_4) = node_5_entry {
                let node_4_entry = &mut node_4.data[bit_index_3];
                if let InternalData::Tile(..) = node_4_entry {
                    *node_4_entry = InternalData::Node(Box::new(<N3<ValueType>>::new()));
                }

                node_4.child_mask[bit_index_3 >> 6] |= 1 << (bit_index_3 & (64 - 1));
                if let InternalData::Node(node_3) = node_4_entry {
                    node_3.value_mask[bit_index_0 >> 6] |= 1 << (bit_index_0 & (64 - 1));
                    node_3.data[bit_index_0] = LeafData::Value(v);
                }
            } else {
                panic!("Unreachable")
            }
        } else {
            panic!("Unreachable")
        }
    }

    pub fn get_voxel(&self, p: [u32; 3]) -> VdbEndpoint<&ValueType> {
        let root_key = self.root.root_key_from_coords(p);

        let Some(root_data) = self.root.map.get(&root_key) else { return VdbEndpoint::Bkgr(&self.root.background) };
        match root_data {
            RootData::Tile(value, _) => VdbEndpoint::Root(value),
            RootData::Node(node5) => {
                let bit_index_4 = <N5<ValueType>>::bit_index_from_coords(p);
                let node5_data = &node5.data[bit_index_4];
                match node5_data {
                    InternalData::Tile(value) => VdbEndpoint::Innr(value, 5),
                    InternalData::Node(node4) => {
                        let bit_index_3 = <N4<ValueType>>::bit_index_from_coords(p);
                        let node4_data = &node4.data[bit_index_3];
                        match node4_data {
                            InternalData::Tile(value) => VdbEndpoint::Innr(value, 4),
                            InternalData::Node(node3) => {
                                let bit_index_0 = <N3<ValueType>>::bit_index_from_coords(p);
                                let node3_data = &node3.data[bit_index_0];
                                match node3_data {
                                    LeafData::Offset(offset) => VdbEndpoint::Offs(*offset),
                                    LeafData::Value(value) => VdbEndpoint::Leaf(value),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::*;

    #[test]
    fn set_voxel_test() {
        let builder = thread::Builder::new()
            .name("set_voxel_test".into())
            .stack_size(80 * 1024 * 1024); // @HACK to increase stack size of this test
        let handler = builder
            .spawn(|| {
                let mut vdb = <VDB345<u8>>::new();
                let points = [[0, 0, 0], [123, 78, 3], [34, 123, 46], [102, 79, 28]];
                for (i, &point) in points.iter().enumerate() {
                    vdb.set_voxel(point, i as u8);
                }
                for (i, &point) in points.iter().enumerate() {
                    let VdbEndpoint::Leaf(&res) = vdb.get_voxel(point) else { panic!("Leaf value not found at point {point:?}"); };
                    assert_eq!(i as u8, res);
                }
            })
            .unwrap();
        handler.join().unwrap_or_else(|_| panic!("Test Failed"));
    }
}
