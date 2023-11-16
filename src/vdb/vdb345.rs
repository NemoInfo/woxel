use cgmath::Vector3;
use itertools::Itertools;

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

const N5DIM: usize = <N5<u32>>::DIM as usize;
const N4DIM: usize = <N4<u32>>::DIM as usize;
const N3DIM: usize = <N3<u32>>::DIM as usize;

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

    pub fn origins(&self) -> Vec<[i32; 3]> {
        let mut origins = vec![];
        for (origin, root_data) in self.root.map.iter().sorted_by_key(|(key, _)| *key) {
            if let RootData::Node(_) = root_data {
                origins.push(*origin);
            }
        }

        origins
    }

    pub fn masks(
        &self,
    ) -> (
        Vec<[u32; 32 * 32 * 32 / 32]>,
        Vec<[u32; 32 * 32 * 32 / 32]>,
        Vec<[u32; 16 * 16 * 16 / 32]>,
        Vec<[u32; 16 * 16 * 16 / 32]>,
        Vec<[u32; 8 * 8 * 8 / 32]>,
    ) {
        let mut n5_kids = vec![];
        let mut n5_vals = vec![];
        let mut n4_kids = vec![];
        let mut n4_vals = vec![];
        let mut n3_vals = vec![];

        for (_origin, root_data) in self.root.map.iter().sorted_by_key(|(key, _)| *key) {
            let RootData::Node(node5) = root_data else {
                continue;
            };

            for node5_data in node5.data.iter() {
                let InternalData::Node(node4) = node5_data else {
                    continue;
                };

                for node4_data in node4.data.iter() {
                    let InternalData::Node(node3) = node4_data else {
                        continue;
                    };

                    n3_vals.push(node3.value_mask);
                }

                n4_vals.push(node4.value_mask);
                n4_kids.push(node4.child_mask);
            }

            n5_vals.push(node5.value_mask);
            n5_kids.push(node5.child_mask);
        }

        (n5_kids, n5_vals, n4_kids, n4_vals, n3_vals)
    }

    pub fn atlas(&self) -> [Vec<Vec<Vec<ValueType>>>; 3] {
        let [count_n5, count_n4, count_n3] = self.count_nodes();

        let n5_atlas_dim = closest_power_of_3(count_n5);
        let n4_atlas_dim = closest_power_of_3(count_n4);
        let n3_atlas_dim = closest_power_of_3(count_n3);

        let mut n5_atlas =
            vec![
                vec![vec![ValueType::zeroed(); N5DIM * n5_atlas_dim]; N5DIM * n5_atlas_dim];
                N5DIM * n5_atlas_dim
            ];
        let mut n4_atlas =
            vec![
                vec![vec![ValueType::zeroed(); N4DIM * n4_atlas_dim]; N4DIM * n4_atlas_dim];
                N4DIM * n4_atlas_dim
            ];
        let mut n3_atlas =
            vec![
                vec![vec![ValueType::zeroed(); N3DIM * n3_atlas_dim]; N3DIM * n3_atlas_dim];
                N3DIM * n3_atlas_dim
            ];

        let mut n5_idx: usize = 0;
        let mut n4_idx: usize = 0;
        let mut n3_idx: usize = 0;

        for (_origin, root_data) in self.root.map.iter().sorted_by_key(|(key, _)| *key) {
            let RootData::Node(node5) = root_data else {
                // TODO: handle node5 tiles
                continue;
            };
            let n5_atlas_origin: Vector3<usize> =
                origin_from_idx(n5_idx, n5_atlas_dim) * <N5<ValueType>>::DIM as usize;

            for (offset, node5_data) in node5.data.iter().enumerate() {
                let n5_data_rel: Vector3<usize> = <N5<ValueType>>::offset_to_relative(offset)
                    .map(|c| c as usize)
                    .into();

                let n5_atlas_data_pos = n5_atlas_origin + n5_data_rel;

                let InternalData::Node(node4) = node5_data else {
                    let &InternalData::Tile(node4_tile) = node5_data else {
                        unreachable!();
                    };
                    n5_atlas[n5_atlas_data_pos.x][n5_atlas_data_pos.y][n5_atlas_data_pos.z] =
                        node4_tile;
                    continue;
                };
                let n4_atlas_origin: Vector3<usize> =
                    origin_from_idx(n4_idx, n4_atlas_dim) * <N4<ValueType>>::DIM as usize;

                for (offset, node4_data) in node4.data.iter().enumerate() {
                    let n4_data_rel: Vector3<usize> = <N4<ValueType>>::offset_to_relative(offset)
                        .map(|c| c as usize)
                        .into();

                    let n4_atlas_data_pos = n4_atlas_origin + n4_data_rel;

                    let InternalData::Node(node3) = node4_data else {
                        let &InternalData::Tile(node3_tile) = node4_data else {
                            unreachable!();
                        };
                        n4_atlas[n4_atlas_data_pos.x][n4_atlas_data_pos.y][n4_atlas_data_pos.z] =
                            node3_tile;
                        continue;
                    };
                    let n3_atlas_origin: Vector3<usize> =
                        origin_from_idx(n3_idx, n3_atlas_dim) * <N3<ValueType>>::DIM as usize;

                    for (offset, node3_data) in node3.data.iter().enumerate() {
                        let n3_data_rel: Vector3<usize> =
                            <N3<ValueType>>::offset_to_relative(offset)
                                .map(|c| c as usize)
                                .into();

                        let n3_atlas_data_pos = n3_atlas_origin + n3_data_rel;

                        n3_atlas[n3_atlas_data_pos.x][n3_atlas_data_pos.y][n3_atlas_data_pos.z] =
                            match node3_data {
                                &LeafData::Value(value) => value,
                                &LeafData::Offset(offset) => {
                                    ValueType::from_4_le_bytes((offset as u32).to_le_bytes())
                                }
                            };
                    }
                    n4_atlas[n4_atlas_data_pos.x][n4_atlas_data_pos.y][n4_atlas_data_pos.z] =
                        ValueType::from_4_le_bytes((n3_idx as u32).to_le_bytes());
                    // HACK: Handle if cast to u32 overflows !!
                    n3_idx += 1;
                }
                n5_atlas[n5_atlas_data_pos.x][n5_atlas_data_pos.y][n5_atlas_data_pos.z] =
                    ValueType::from_4_le_bytes((n4_idx as u32).to_le_bytes());
                // HACK: Handle if cast to u32 overflows !!
                n4_idx += 1;
            }

            n5_idx += 1;
        }

        [n5_atlas, n4_atlas, n3_atlas]
    }

    pub fn count_nodes(&self) -> [usize; 3] {
        let mut count: [usize; 3] = [0, 0, 0];
        for (_, root_data) in self.root.map.iter() {
            let RootData::Node(node5) = root_data else {
                continue;
            };
            count[0] += 1;
            for node5_data in node5.data.iter() {
                let InternalData::Node(node4) = node5_data else {
                    continue;
                };
                count[1] += 1;
                for node4_data in node4.data.iter() {
                    let InternalData::Node(_) = node4_data else {
                        continue;
                    };
                    count[2] += 1;
                }
            }
        }
        count
    }
}

fn closest_power_of_3(n: usize) -> usize {
    let mut i = 0;
    while i * i * i < n {
        i += 1;
    }
    i
}

fn origin_from_idx(idx: usize, dim: usize) -> Vector3<usize> {
    (idx % dim, (idx / dim) % dim, idx / (dim * dim)).into()
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
