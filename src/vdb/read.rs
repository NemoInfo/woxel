use std::{
    collections::HashMap,
    io::{Read, Seek, SeekFrom},
    string::FromUtf8Error,
};

use bitvec::prelude::*;
use blosc_src::blosc_cbuffer_sizes;
use bytemuck::{bytes_of_mut, cast_slice_mut, Pod, Zeroable};
use byteorder::{LittleEndian, ReadBytesExt};
use cgmath::{Vector3, Zero};
use half::f16;
use log::{trace, warn};

use crate::vdb::{Compression, Node, NodeHeader, NodeMetaData, Root345, RootData, N5};

use super::transform::Map;
use super::{ArchiveHeader, GridDescriptor, Metadata, MetadataValue, VDB345};

type Result<T> = std::result::Result<T, ErrorKind>;

const OPENVDB_MIN_SUPPORTED_VERSION: u32 = OPENVDB_FILE_VERSION_BOOST_UUID;

const OPENVDB_FILE_VERSION_PER_GRID_COMPRESSION: u32 = 223;
pub const OPENVDB_FILE_VERSION_SELECTIVE_COMPRESSION: u32 = 220;
pub const OPENVDB_FILE_VERSION_NODE_MASK_COMPRESSION: u32 = 222;
pub const OPENVDB_FILE_VERSION_BOOST_UUID: u32 = 218;

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("Magic bytes mismatched")]
    MagicMismatch,
    #[error("Unsupported VDB file version {0}")]
    UnsupportedVErsion(u32),
    #[error("IoError")]
    IoError(#[from] std::io::Error),
    #[error("Utf8Error")]
    Utf8Error(#[from] FromUtf8Error),
    #[error("Invalid compression {0}")]
    InvalidCompression(u32),
    #[error("Invaild grid name {0}")]
    InvalidGridName(String),
    #[error("File bbox min not provided")]
    FileBboxMinNotFound,
    #[error("Invalid node metadata entry (u8)")]
    InvalidNodeMetadata(u8),
    #[error("Unsupported Blosc format")]
    UnsupportedBloscFormat,
    #[error("Invalid Blsoc data")]
    InvalidBloscData,
}

pub struct VdbReader<R: Read + Seek> {
    reader: R,
    pub header: ArchiveHeader,
    pub grid_descriptors: HashMap<String, GridDescriptor>,
}

impl<R: Read + Seek> VdbReader<R> {
    pub fn new(mut reader: R) -> Result<Self> {
        let magic = reader.read_u64::<LittleEndian>()?;
        if magic != 0x56444220 {
            return Err(ErrorKind::MagicMismatch);
        }

        let file_version = reader.read_u32::<LittleEndian>()?;
        if file_version < OPENVDB_MIN_SUPPORTED_VERSION {
            return Err(ErrorKind::UnsupportedVErsion(file_version));
        }

        let library_major = reader.read_u32::<LittleEndian>()?;
        let library_minor = reader.read_u32::<LittleEndian>()?;

        let has_grid_offsets = reader.read_u8()? != 0;

        let mut compression = if file_version < OPENVDB_FILE_VERSION_PER_GRID_COMPRESSION {
            // Prior to the introduction of Blosc, ZLIB was the default compression scheme.
            Compression::ZIP | Compression::ACTIVE_MASK
        } else {
            // From version 223 on, compression information is stored per grid
            Compression::DEFAULT_COMPRESSION
        };

        if (OPENVDB_FILE_VERSION_SELECTIVE_COMPRESSION..OPENVDB_FILE_VERSION_NODE_MASK_COMPRESSION)
            .contains(&file_version)
        {
            let is_compressed = reader.read_u8()? == 1;
            if is_compressed {
                compression = Compression::ZIP;
            } else {
                compression = Compression::NONE;
            }
        }

        let uuid = read_string(&mut reader, 36)?;

        let meta_data = Self::read_metadata(&mut reader)?;

        let grid_number = reader.read_u32::<LittleEndian>()?;

        let header = ArchiveHeader {
            file_version,
            library_major,
            library_minor,
            uuid,
            has_grid_offsets,
            compression,
            grid_number,
            meta_data,
        };

        let grid_descriptors = Self::read_grid_descriptors(&mut reader, &header)?;

        Ok(Self {
            reader,
            header,
            grid_descriptors,
        })
    }

    pub fn read_vdb345_grid<T: From4LeBytes + std::fmt::Debug + Pod>(
        &mut self,
        name: &str,
    ) -> Result<VDB345<T>> {
        let grid_descriptor = self.grid_descriptors.get(name).cloned();
        let grid_descriptor =
            grid_descriptor.ok_or_else(|| ErrorKind::InvalidGridName(name.to_owned()))?;
        grid_descriptor.seek_to_grid(&mut self.reader)?;

        if self.header.file_version >= OPENVDB_FILE_VERSION_NODE_MASK_COMPRESSION {
            let _: Compression = self.reader.read_u32::<LittleEndian>()?.try_into()?;
        }
        let _ = Self::read_metadata(&mut self.reader)?;

        let transform = Self::read_transform(&mut self.reader)?;

        let tree = self.read_tree_topology::<T>(&grid_descriptor)?;

        todo!();
    }

    fn read_transform(reader: &mut R) -> Result<Map> {
        let transform_name = read_len_string(reader)?;

        Ok(match transform_name.as_str() {
            "UniformScaleMap" => Map::UniformScaleMap {
                scale_values: read_vec3d(reader)?,
                voxel_size: read_vec3d(reader)?,
                scale_values_inverse: read_vec3d(reader)?,
                inv_scale_sqr: read_vec3d(reader)?,
                inv_twice_scale: read_vec3d(reader)?,
            },
            "UniformScaleTranslateMap" | "ScaleTranslateMap" => Map::ScaleTranslateMap {
                translation: read_vec3d(reader)?,
                scale_values: read_vec3d(reader)?,
                voxel_size: read_vec3d(reader)?,
                scale_values_inverse: read_vec3d(reader)?,
                inv_scale_sqr: read_vec3d(reader)?,
                inv_twice_scale: read_vec3d(reader)?,
            },
            _ => panic!("Not suppored transform type {transform_name}"),
        })
    }

    fn read_grid_descriptors(
        reader: &mut R,
        header: &ArchiveHeader,
    ) -> Result<HashMap<String, GridDescriptor>> {
        // Guaranteed by minimum file version
        assert!(header.has_grid_offsets);

        let mut grid_descriptors = HashMap::new();

        for _ in 0..header.grid_number {
            let name = read_len_string(reader)?;
            let grid_type = read_len_string(reader)?;
            let instance_parent = read_len_string(reader)?;

            let grid_pos = reader.read_u64::<LittleEndian>()?;
            let block_pos = reader.read_u64::<LittleEndian>()?;
            let end_pos = reader.read_u64::<LittleEndian>()?;

            let mut grid_descriptor = GridDescriptor {
                name: name.clone(),
                grid_type,
                instance_parent,
                grid_pos,
                block_pos,
                end_pos,
                compression: header.compression,
                meta_data: Default::default(),
                bbox_min: Vector3::zero(),
            };

            if header.file_version >= OPENVDB_FILE_VERSION_NODE_MASK_COMPRESSION {
                grid_descriptor.compression = reader.read_u32::<LittleEndian>()?.try_into()?;
            }

            grid_descriptor.meta_data = Self::read_metadata(reader)?;

            let Some(MetadataValue::Vec3i(bbox_min)) = grid_descriptor.meta_data.0.get("file_bbox_min") else { return Err(ErrorKind::FileBboxMinNotFound) };

            grid_descriptor.bbox_min = *bbox_min;

            assert!(
                grid_descriptors
                    .insert(name.clone(), grid_descriptor)
                    .is_none(),
                "Gird {name} already exists"
            );

            reader.seek(SeekFrom::Start(end_pos))?;
        }

        Ok(grid_descriptors)
    }

    fn read_metadata(reader: &mut R) -> Result<Metadata> {
        let meta_data_number = reader.read_u32::<LittleEndian>()?;
        let mut meta_data = Metadata::default();

        for _ in 0..meta_data_number {
            let name = read_len_string(reader)?;
            let meta_type = read_len_string(reader)?;

            let meta_len = reader.read_u32::<LittleEndian>()? as usize;
            meta_data.0.insert(
                name,
                match meta_type.as_str() {
                    "string" => MetadataValue::String(read_string(reader, meta_len)?),
                    "bool" => {
                        let val = reader.read_u8()?;
                        MetadataValue::Bool(val == 1)
                    }
                    "int32" => {
                        let val = reader.read_i32::<LittleEndian>()?;
                        MetadataValue::I32(val)
                    }
                    "int64" => {
                        let val = reader.read_i64::<LittleEndian>()?;
                        MetadataValue::I64(val)
                    }
                    "float" => {
                        let val = reader.read_f32::<LittleEndian>()?;
                        MetadataValue::Float(val)
                    }
                    "vec3i" => {
                        let val = read_vec3i(reader)?;
                        MetadataValue::Vec3i(val)
                    }
                    name => {
                        let mut data = vec![0u8; meta_len];
                        reader.read_exact(&mut data)?;

                        warn!("Unknown metadata value {}", name);

                        MetadataValue::Unknown {
                            name: name.to_owned(),
                            data,
                        }
                    }
                },
            );
        }

        trace!("Metadata:");
        for (name, value) in &meta_data.0 {
            trace!("{name} {value:?}");
        }

        Ok(meta_data)
    }

    fn read_tree_topology<T: From4LeBytes + std::fmt::Debug + Pod>(
        &mut self,
        grid_descriptor: &GridDescriptor,
    ) -> Result<VDB345<T>> {
        let buffer_count = self.reader.read_u32::<LittleEndian>()?;
        if buffer_count != 1 {
            todo!("Multi-buffer trees not implemented");
        }

        dbg!(self.reader.stream_position()?);

        // let root_node_background =
        //     T::from_4_le_bytes(reader.read_u32::<LittleEndian>()?.to_le_bytes());
        // @TODO: What is the logic here for half_float? this takes up 4 bytes?
        let rn_bytes = self.reader.read_u32::<LittleEndian>()?;
        let root_node_background = T::from_4_le_bytes(rn_bytes.to_le_bytes());

        let number_of_tiles = self.reader.read_u32::<LittleEndian>()?;
        let number_of_node5s = self.reader.read_u32::<LittleEndian>()?;
        dbg!(&root_node_background);
        dbg!(&number_of_tiles);
        dbg!(&number_of_node5s);

        let mut node5s_entry = vec![];

        // Iterate Node5 tiles
        for _ in 0..number_of_tiles {
            let origin: [i32; 3] = read_vec3i(&mut self.reader)?.into();
            let uorigin: [u32; 3] = grid_descriptor.world_to_u(origin);
            let root_key = <Root345<T>>::root_key_from_coords(uorigin);

            let value_bytes = self.reader.read_u32::<LittleEndian>()?;
            let value = T::from_4_le_bytes(value_bytes.to_le_bytes());
            let active = self.reader.read_u8()? == 1;

            let node5_tile = RootData::Tile::<T, N5<T>>(value, active);
            node5s_entry.push((root_key, node5_tile));
        }

        // Iterate Node5 Children
        for _ in 0..number_of_node5s {
            let origin: [i32; 3] = read_vec3i(&mut self.reader)?.into();

            let node_5_header = self.read_internal_node_header::<T, N5<T>>(&grid_descriptor)?;

            let mut node_5 = <N5<T>>::new();

            assert_eq!(
                node_5_header.child_mask.len(),
                <N5<T>>::SIZE * 64,
                "Read mask is not the same length as expected mask"
            );

            let slice: &[u64] = node_5_header.child_mask.as_raw_slice();
            // let array = [0u64; <N5<T>>::SIZE / 64];

            todo!()
        }

        dbg!(node5s_entry);

        todo!()
    }

    fn read_internal_node_header<T: Pod, N: Node>(
        &mut self,
        grid_descriptor: &GridDescriptor,
    ) -> Result<NodeHeader<T>> {
        let mut child_mask = bitvec![u64, Lsb0; 0; N::SIZE];
        let mut value_mask = bitvec![u64, Lsb0; 0; N::SIZE];
        self.reader
            .read_u64_into::<LittleEndian>(child_mask.as_raw_mut_slice())?;
        self.reader
            .read_u64_into::<LittleEndian>(value_mask.as_raw_mut_slice())?;

        let size = if self.header.file_version < OPENVDB_FILE_VERSION_NODE_MASK_COMPRESSION {
            child_mask.count_zeros()
        } else {
            N::SIZE
        };

        let data = self.read_compressed(grid_descriptor, size, value_mask.as_bitslice())?;

        Ok(NodeHeader {
            child_mask,
            value_mask,
            data,
            log_2_dim: N::LOG2_D,
        })
    }

    fn read_compressed<T: Pod>(
        &mut self,
        grid_descriptor: &GridDescriptor,
        size: usize,
        value_mask: &BitSlice<u64>,
    ) -> Result<Vec<T>> {
        let mut meta_data: NodeMetaData = NodeMetaData::NoMaskAndAllVals;
        if self.header.file_version >= OPENVDB_FILE_VERSION_NODE_MASK_COMPRESSION {
            meta_data = self.reader.read_u8()?.try_into()?;
        }

        let mut inactive_val0 = T::zeroed();
        let mut inactive_val1 = T::zeroed();
        match meta_data {
            NodeMetaData::MaskAndOneInactiveVal | NodeMetaData::NoMaskAndOneInactiveVal => {
                self.reader.read_exact(bytes_of_mut(&mut inactive_val0))?;
            }
            NodeMetaData::MaskAndTwoInactiveVals => {
                self.reader.read_exact(bytes_of_mut(&mut inactive_val0))?;
                self.reader.read_exact(bytes_of_mut(&mut inactive_val1))?;
            }
            _ => {}
        }

        let mut selection_mask = bitvec![u64, Lsb0; 0; size];

        if meta_data == NodeMetaData::MaskAndNoInactiveVals
            || meta_data == NodeMetaData::MaskAndOneInactiveVal
            || meta_data == NodeMetaData::MaskAndTwoInactiveVals
        {
            self.reader
                .read_u64_into::<LittleEndian>(selection_mask.as_raw_mut_slice())?;
        }

        let count = if grid_descriptor
            .compression
            .contains(Compression::ACTIVE_MASK)
            && meta_data != NodeMetaData::NoMaskAndAllVals
            && self.header.file_version >= OPENVDB_FILE_VERSION_NODE_MASK_COMPRESSION
        {
            value_mask.count_ones()
        } else {
            size
        };

        let data: Vec<T> = if grid_descriptor.meta_data.is_half_float()
            && std::any::TypeId::of::<T>() == std::any::TypeId::of::<f32>()
        {
            let data = self.read_compressed_data::<f16>(grid_descriptor, count)?;
            bytemuck::cast_vec(data.into_iter().map(f16::to_f32).collect::<Vec<f32>>())
        } else if !grid_descriptor.meta_data.is_half_float() {
            let data = self.read_compressed_data::<f32>(grid_descriptor, count)?;
            bytemuck::cast_vec(data.into_iter().map(f16::from_f32).collect::<Vec<_>>())
        } else {
            self.read_compressed_data(grid_descriptor, count)?
        };

        Ok(
            if grid_descriptor
                .compression
                .contains(Compression::ACTIVE_MASK)
                && data.len() != size
            {
                trace!("Expanding active maska data {} to {}", data.len(), size);

                let mut expanded = vec![T::zeroed(); size];
                let mut read_idx = 0;
                for dest_idx in 0..size {
                    expanded[dest_idx] = if value_mask[dest_idx] {
                        let v = data[read_idx];
                        read_idx += 1;
                        v
                    } else if selection_mask[dest_idx] {
                        inactive_val1
                    } else {
                        inactive_val0
                    }
                }
                expanded
            } else {
                data
            },
        )
    }

    fn read_compressed_data<T: Pod>(
        &mut self,
        grid_descriptor: &GridDescriptor,
        count: usize,
    ) -> Result<Vec<T>> {
        Ok(match grid_descriptor.compression {
            c if c.contains(Compression::BLOSC) => {
                let num_compressed_bytes = self.reader.read_i64::<LittleEndian>()?;
                let compressed_count = num_compressed_bytes / std::mem::size_of::<T>() as i64;

                trace!("Reading blosc data, {} bytes", num_compressed_bytes);
                if num_compressed_bytes <= 0 {
                    let mut data = vec![T::zeroed(); (-compressed_count) as usize];
                    self.reader.read_exact(cast_slice_mut(&mut data))?;
                    assert_eq!(-compressed_count as usize, count);
                    data
                } else {
                    let mut blosc_data = vec![0u8; num_compressed_bytes as usize];
                    self.reader.read_exact(&mut blosc_data)?;
                    if count > 0 {
                        let mut nbytes: usize = 0;
                        let mut cbytes: usize = 0;
                        let mut blocksize: usize = 0;
                        unsafe {
                            blosc_cbuffer_sizes(
                                blosc_data.as_ptr().cast(),
                                &mut nbytes,
                                &mut cbytes,
                                &mut blocksize,
                            )
                        };
                        if nbytes == 0 {
                            return Err(ErrorKind::UnsupportedBloscFormat);
                        }
                        let dest_size = nbytes / std::mem::size_of::<T>();
                        let mut dest: Vec<T> = vec![Zeroable::zeroed(); dest_size];
                        let error = unsafe {
                            blosc_src::blosc_decompress_ctx(
                                blosc_data.as_ptr().cast(),
                                dest.as_mut_ptr().cast(),
                                nbytes,
                                1,
                            )
                        };
                        if error < 1 {
                            return Err(ErrorKind::InvalidBloscData);
                        }
                        dest
                    } else {
                        trace!(
                            "Skipping blosc decompression because of a {}-count read",
                            count
                        );
                        vec![T::zeroed(); 0]
                    }
                }
            }
            c if c.contains(Compression::ZIP) => {
                let num_zipped_bytes = self.reader.read_i64::<LittleEndian>()?;
                let compressed_count = num_zipped_bytes / std::mem::size_of::<T>() as i64;

                trace!("Reading zipped data, {} bytes", num_zipped_bytes);
                if num_zipped_bytes <= 0 {
                    let mut data = vec![T::zeroed(); (-compressed_count) as usize];
                    self.reader.read_exact(cast_slice_mut(&mut data))?;
                    data
                } else {
                    let mut zipped_data = vec![0u8; num_zipped_bytes as usize];
                    self.reader.read_exact(&mut zipped_data)?;

                    let mut zip_reader = flate2::read::ZlibDecoder::new(zipped_data.as_slice());
                    let mut data = vec![T::zeroed(); count];
                    zip_reader.read_exact(cast_slice_mut(&mut data))?;
                    data
                }
            }
            _ => {
                trace!("Reading uncompressed data, {} elements", count);

                let mut data = vec![T::zeroed(); count];
                self.reader.read_exact(cast_slice_mut(&mut data))?;
                data
            }
        })
    }
}

pub trait From4LeBytes {
    fn from_4_le_bytes(array: [u8; 4]) -> Self;
}

impl From4LeBytes for f32 {
    fn from_4_le_bytes(array: [u8; 4]) -> Self {
        f32::from_le_bytes(array)
    }
}

impl From4LeBytes for f16 {
    fn from_4_le_bytes(array: [u8; 4]) -> Self {
        f16::from_le_bytes([array[0], array[1]])
    }
}

impl From4LeBytes for u8 {
    fn from_4_le_bytes(array: [u8; 4]) -> Self {
        u8::from_le_bytes([array[0]])
    }
}

impl From4LeBytes for u16 {
    fn from_4_le_bytes(array: [u8; 4]) -> Self {
        u16::from_le_bytes([array[0], array[1]])
    }
}

impl TryFrom<u32> for Compression {
    type Error = ErrorKind;

    fn try_from(v: u32) -> Result<Compression> {
        Self::from_bits(v).ok_or(ErrorKind::InvalidCompression(v))
    }
}

fn read_len_string<R: Read + Seek>(reader: &mut R) -> Result<String> {
    let len = reader.read_u32::<LittleEndian>()? as usize;
    read_string(reader, len)
}

fn read_string<R: Read + Seek>(reader: &mut R, len: usize) -> Result<String> {
    let mut buf = vec![0; len];
    reader.read_exact(&mut buf)?;
    let string = String::from_utf8(buf.to_vec())?;
    Ok(string)
}

fn read_vec3i<R: Read + Seek>(reader: &mut R) -> Result<Vector3<i32>> {
    let x = reader.read_i32::<LittleEndian>()?;
    let y = reader.read_i32::<LittleEndian>()?;
    let z = reader.read_i32::<LittleEndian>()?;

    Ok(Vector3 { x, y, z })
}

fn read_vec3d<R: Read + Seek>(reader: &mut R) -> Result<Vector3<f64>> {
    let x = reader.read_f64::<LittleEndian>()?;
    let y = reader.read_f64::<LittleEndian>()?;
    let z = reader.read_f64::<LittleEndian>()?;

    Ok(Vector3 { x, y, z })
}

impl TryFrom<u8> for NodeMetaData {
    type Error = ErrorKind;

    fn try_from(v: u8) -> Result<NodeMetaData> {
        Ok(match v {
            0 => Self::NoMaskOrInactiveVals,
            1 => Self::NoMaskAndMinusBg,
            2 => Self::NoMaskAndOneInactiveVal,
            3 => Self::MaskAndNoInactiveVals,
            4 => Self::MaskAndOneInactiveVal,
            5 => Self::MaskAndTwoInactiveVals,
            6 => Self::NoMaskAndAllVals,
            _ => return Err(ErrorKind::InvalidNodeMetadata(v)),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use super::*;

    #[test]
    fn test_read_utahteapot() {
        let f = std::fs::File::open("assets/utahteapot.vdb").unwrap();
        let b = BufReader::new(f);

        let mut vdb_reader = VdbReader::new(b).unwrap();
        dbg!(&vdb_reader.header);
        dbg!(&vdb_reader.grid_descriptors);

        let vdb = vdb_reader.read_vdb345_grid::<f16>("ls_utahteapot").unwrap();

        todo!();
    }
}
