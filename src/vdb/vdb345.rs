use crate::vdb::data_structure::*;

use super::VdbValueType;

pub type N3<ValueType> = LeafNode<ValueType, 3>;
pub type N4<ValueType> = InternalNode<ValueType, N3<ValueType>, 4>;
pub type N5<ValueType> = InternalNode<ValueType, N4<ValueType>, 5>;
pub type Root345<ValueType> = RootNode<ValueType, N5<ValueType>>;
pub type VDB345<ValueType> = VDB<ValueType, N5<ValueType>>;

pub type N3Cube<ValueType> = [[[ValueType; 1 << 3]; 1 << 3]; 1 << 3];
pub type N4Cube<ValueType> = [[[ValueType; 1 << 4]; 1 << 4]; 1 << 4];
pub type N5Cube<ValueType> = [[[ValueType; 1 << 5]; 1 << 5]; 1 << 5];

impl<'a, ValueType> VDB345<ValueType>
where
    ValueType: VdbValueType,
{
    /// Sets the value `v` of a single voxel in the VDB at point `p`.
    pub fn set_voxel(&mut self, p: GlobalCoordinates, v: ValueType) {
        let root_key = <Root345<ValueType>>::root_key_from_coords(p);
        let bit_index_4 = <N5<ValueType>>::global_to_offset(p);
        let bit_index_3 = <N4<ValueType>>::global_to_offset(p);
        let bit_index_0 = <N3<ValueType>>::global_to_offset(p);

        let root_entry = self
            .root
            .map
            .entry(root_key)
            .or_insert_with(|| RootData::Node(Box::new(<N5<ValueType>>::new(p))));

        if let RootData::Tile(..) = root_entry {
            *root_entry = RootData::Node(Box::new(<N5<ValueType>>::new(p)));
        }

        let RootData::Node(node_5) = root_entry else {
            unreachable!()
        };

        let node_5_entry = &mut node_5.data[bit_index_4];
        if let InternalData::Tile(..) = node_5_entry {
            *node_5_entry = InternalData::Node(Box::new(<N4<ValueType>>::new(p)));
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
            unreachable!();
        }
    }

    /// Returns the value of a single voxel in the VDB at point `p`.
    pub fn get_voxel(&self, p: GlobalCoordinates) -> VdbEndpoint<&ValueType> {
        let root_key = <Root345<ValueType>>::root_key_from_coords(p);

        let Some(root_data) = self.root.map.get(&root_key) else {
            return VdbEndpoint::Bkgr(&self.root.background);
        };
        let RootData::Node(node5) = root_data else {
            let RootData::Tile(value, _) = root_data else {
                unreachable!()
            };
            return VdbEndpoint::Root(value);
        };

        let bit_index_4 = <N5<ValueType>>::global_to_offset(p);
        let node5_data = &node5.data[bit_index_4];
        let InternalData::Node(node4) = node5_data else {
            let InternalData::Tile(value) = node5_data else {
                unreachable!()
            };
            return VdbEndpoint::Innr(value, 5);
        };

        let bit_index_3 = <N4<ValueType>>::global_to_offset(p);
        let node4_data = &node4.data[bit_index_3];
        let InternalData::Node(node3) = node4_data else {
            let InternalData::Tile(value) = node4_data else {
                unreachable!()
            };
            return VdbEndpoint::Innr(value, 4);
        };

        let bit_index_0 = <N3<ValueType>>::global_to_offset(p);
        let node3_data = &node3.data[bit_index_0];
        match node3_data {
            LeafData::Offset(offset) => VdbEndpoint::Offs(*offset),
            LeafData::Value(value) => VdbEndpoint::Leaf(value),
        }
    }

    pub fn node_vecs(
        &self,
    ) -> (
        Vec<N5Cube<ValueType>>,
        Vec<N4Cube<ValueType>>,
        Vec<N3Cube<ValueType>>,
    ) {
        let (mut n5s, mut n4s, mut n3s) = (vec![], vec![], vec![]);

        for (origin, root_data) in self.root.map.iter() {
            let RootData::Node(node5) = root_data else {
                continue;
            };

            let mut n5_cube: N5Cube<ValueType> = [[[self.root.background; 1 << 5]; 1 << 5]; 1 << 5];

            for (offset, node5_data) in node5.data.iter().enumerate() {
                let InternalData::Node(node4) = node5_data else {
                    continue;
                };
                let (x, y, z): (usize, usize, usize) = <N5<ValueType>>::offset_to_relative(offset)
                    .map(|c| c as usize)
                    .into();

                let mut n4_cube: N4Cube<ValueType> =
                    [[[self.root.background; 1 << 4]; 1 << 4]; 1 << 4];

                for (offset, node4_data) in node4.data.iter().enumerate() {
                    let InternalData::Node(node3) = node4_data else {
                        continue;
                    };
                    let (x, y, z): (usize, usize, usize) =
                        <N4<ValueType>>::offset_to_relative(offset)
                            .map(|c| c as usize)
                            .into();

                    let mut n3_cube: N3Cube<ValueType> =
                        [[[self.root.background; 1 << 3]; 1 << 3]; 1 << 3];

                    for (offset, node3_data) in node3.data.iter().enumerate() {
                        let &LeafData::Value(voxel) = node3_data else {
                            continue;
                            // TODO: If SDF we should encode this somehow
                            // We can just store this same as a voxel and send the masks in a separate binding to the gpu
                        };

                        let (x, y, z): (usize, usize, usize) =
                            <N3<ValueType>>::offset_to_relative(offset)
                                .map(|c| c as usize)
                                .into();

                        n3_cube[x][y][z] = voxel;
                    }

                    let n3_idx = n3s.len() as u32;
                    n3s.push(n3_cube);
                    n4_cube[x][y][z] = ValueType::from_4_le_bytes(n3_idx.to_le_bytes());
                }

                let n4_idx = n4s.len() as u32;
                n4s.push(n4_cube);
                n5_cube[x][y][z] = ValueType::from_4_le_bytes(n4_idx.to_le_bytes());
            }

            n5s.push(n5_cube);
        }

        (n5s, n4s, n3s)
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::*;

    #[test]
    fn set_get_voxel_test() {
        let builder = thread::Builder::new()
            .name("set_voxel_test".into())
            .stack_size(80 * 1024 * 1024); // @HACK to increase stack size of this test
        let handler = builder
            .spawn(|| {
                let mut vdb = <VDB345<u8>>::new();
                let points = [[0, 0, 0], [123, 78, 3], [34, 123, 46], [102, 79, 28]];
                for (i, &point) in points.iter().enumerate() {
                    vdb.set_voxel(point.into(), i as u8);
                }
                for (i, &point) in points.iter().enumerate() {
                    let VdbEndpoint::Leaf(&res) = vdb.get_voxel(point.into()) else {
                        panic!("Leaf value not found at point {point:?}");
                    };
                    assert_eq!(i as u8, res);
                }
            })
            .unwrap();
        handler.join().unwrap_or_else(|_| panic!("Test Failed"));
    }
}
