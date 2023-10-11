use bytemuck::Pod;
pub trait VdbValueType = From4LeBytes + Pod + Copy + CopyBytesToU32;

mod data_structure;
pub use data_structure::*;

mod vdb345;
pub use vdb345::*;

mod write;
pub use write::*;

mod read;
pub use read::*;

mod transform;
