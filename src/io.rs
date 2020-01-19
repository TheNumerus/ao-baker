use std::path::PathBuf;
use std::fs::File;
use std::io::{BufReader, Read};

use wavefront_obj::obj;

pub fn read_obj(filename: PathBuf) -> obj::Object {
    let file = File::open(filename).unwrap();
    let mut file_content = String::new();
    let mut reader = BufReader::new(file);
    reader.read_to_string(&mut file_content).unwrap();
    obj::parse(file_content).unwrap().objects[0].to_owned()
}
