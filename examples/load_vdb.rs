use std::io::BufReader;

extern crate woxel;
use woxel::vdb::VdbReader;

fn main() {
    let f = std::fs::File::open("assets/utahteapot.vdb").unwrap();
    let mut vdb_reader = VdbReader::new(BufReader::new(f)).unwrap();
    let vdb = vdb_reader.read_vdb345_grid::<f32>("ls_utahteapot").unwrap();
    vdb.atlas();
}
