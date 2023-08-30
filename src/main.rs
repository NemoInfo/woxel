#![feature(generic_const_exprs)]
#![feature(raw_ref_op)]

mod vdb;

use std::mem::size_of_val;

use crate::vdb::*;

pub fn main() {
    let mut vdb = <VDB345<u8>>::new();
    dbg!(&vdb);
    vdb.set_voxel([123, 78, 3], 7);
    vdb.set_voxel([34, 123, 46], 9);
    vdb.set_voxel([102, 79, 28], 3);
    vdb.set_voxel([0, 0, 0], 3);
    vdb.set_voxel([10002020, 0, 0], 3);
    let mut input = String::new();
    vdb.root.pretty_print(&mut input);
    input = input.replace("woxel::vdb::data_structure::", "");
    println!("{input}");
    println!("Size of VDB = {}", size_of_val(&vdb));
}
