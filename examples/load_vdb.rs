use std::io::BufReader;

extern crate woxel;
use woxel::vdb::VdbReader;

fn main() {
    let f = std::fs::File::open("assets/utahteapot.vdb").unwrap();
    let mut vdb_reader = VdbReader::new(BufReader::new(f)).unwrap();
    let vdb = vdb_reader.read_vdb345_grid::<f32>("ls_utahteapot").unwrap();
    let node_vecs = vdb.node_vecs();
    dbg!(vdb.root.background);
    dbg!(node_vecs.0.len());
    dbg!(node_vecs.1.len());
    dbg!(node_vecs.2.len());
}
