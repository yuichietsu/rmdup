use std::error::Error;
use std::fs;
use std::io::{self, Read};
use std::collections::HashMap;
use cab;
use crc32fast::Hasher;
use crate::archiver;
use tempfile::tempdir;
use fs_extra::dir::{copy, CopyOptions};

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

pub fn remove(container : &str, _path : &str) -> Result<(), Box<dyn Error>> {
    let temp_dir = tempdir()?;
    println!("Temporary directory path: {:?}", temp_dir.path());

    let cabinet = cab::Cabinet::new(fs::File::open(container)?)?;
    let mut c = cab::Cabinet::new(fs::File::open(container)?)?;
    for folder in cabinet.folder_entries() {
        for file in folder.file_entries() {
            let mut r = c.read_file(file.name())?;
            let t = temp_dir.path().to_str().unwrap();
            let p = format!("{}/{}", t, file.name());
            println!("Created {}", p);
            let mut w = fs::File::create(p)?;
            io::copy(&mut r, &mut w)?;
        }
    }

    let options = CopyOptions::new();
    let temp_dir_path = temp_dir.into_path();
    copy(temp_dir_path, "/tmp/cab", &options).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

    println!("moved");

    Ok(())
}
