use std::fs;
use std::io::{self, Read};
use std::collections::HashMap;
use cab;
use crc32fast::Hasher;
use crate::archiver;

pub fn walk(
    file_name : &str,
    map_len   : &mut HashMap<u64, Vec<String>>,
    _map_crc   : &mut HashMap<String, u32>,
) -> Result<(), io::Error> {
    let cab_file = fs::File::open(file_name)?;
    let cabinet = cab::Cabinet::new(cab_file)?;
    for folder in cabinet.folder_entries() {
        for file in folder.file_entries() {
            let len  = file.uncompressed_size() as u64;
            let name = format!("{}\t{}", file_name, file.name());
            archiver::push_map_len(map_len, len, name.as_str());
        }
    }
    Ok(())
}

pub fn crc(container : &str, path : &str) -> Result<u32, io::Error> {
    let mut hasher = Hasher::new();
    let cab_file = fs::File::open(container)?;
    let mut cabinet = cab::Cabinet::new(cab_file)?;
    let mut reader = cabinet.read_file(path)?;
    let mut buffer = [0; 4096];
    loop {
        match reader.read(&mut buffer)? {
            0 => break,
            n => {
                hasher.update(&buffer[..n]);
            }
        }
    }
    Ok(hasher.finalize())
}
