use std::fs;
use std::io::{self, Read};
use std::collections::HashMap;
use cab;
use crc32fast::Hasher;

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
            if let Some(paths) = map_len.get_mut(&len) {
                paths.push(name);
            } else {
                map_len.insert(len, vec![name]);
            }
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
