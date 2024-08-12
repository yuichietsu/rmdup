use zip;
use std::error::Error;
use std::fs;
use std::io;
use std::collections::HashMap;
use crate::archiver;

pub fn walk(
    file_name : &str,
    map_len   : &mut HashMap<u64, Vec<String>>,
    map_crc   : &mut HashMap<String, u32>,
) -> Result<(), io::Error> {
    let zip_file = fs::File::open(file_name)?;
    let reader   = io::BufReader::new(zip_file);
    let mut archive  = zip::ZipArchive::new(reader)?;
    for i in 0..archive.len() {
        let file = archive.by_index(i).unwrap();
        if file.is_file() {
            let len  = file.size();
            let name = format!("{}\t{}", file_name, file.name());
            archiver::push_map_len(map_len, len, name.as_str());
            map_crc.insert(name, file.crc32()); 
        }
    }
    Ok(())
}

pub fn crc(container : &str, path : &str) -> Result<u32, io::Error> {
    let zip_file = fs::File::open(container)?;
    let reader   = io::BufReader::new(zip_file);
    let mut archive  = zip::ZipArchive::new(reader).unwrap();
    let file = archive.by_name(path).unwrap();
    Ok(file.crc32())
}

pub fn remove(container : &str, files : Vec<String>) -> Result<(), Box<dyn Error>> {
    for file in files {
        println!("zip : {} in {}", file, container);
    }
    Ok(())
}
