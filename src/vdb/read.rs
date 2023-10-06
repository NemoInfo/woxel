use std::{
    collections::HashMap,
    io::{Read, Seek, SeekFrom},
    string::FromUtf8Error,
};

use byteorder::{LittleEndian, ReadBytesExt};
use cgmath::{num_traits::FromBytes, Vector3, Zero};
use half::f16;
use log::{trace, warn};

use crate::vdb::{Compression, Node, Root345, RootData, N5};

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

    pub fn read_vdb345_grid<T: From4LeBytes + std::fmt::Debug>(
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

    fn read_tree_topology<T: From4LeBytes + std::fmt::Debug>(
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

            let node_5: Result<N5<T>> = self.read_internal_node_header::<T, N5<T>>();
        }

        dbg!(node5s_entry);

        todo!()
    }

    fn read_internal_node_header<T, N: Node>(&mut self) -> Result<N5<T>> {
        let mut child_mask = vec![0_u64; N::SIZE as usize / 64];
        let mut value_mask = vec![0_u64; N::SIZE as usize / 64];
        self.reader.read_u64_into::<LittleEndian>(&mut child_mask)?;
        self.reader.read_u64_into::<LittleEndian>(&mut value_mask)?;
        todo!();
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
