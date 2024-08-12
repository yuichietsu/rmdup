use std::fs;
use std::error::Error;
use std::io::{self, BufReader, Read};
use std::collections::HashMap;
use crc32fast::Hasher;

use crate::archiver;

pub fn walk(
    dir     : &str,
    map_len : &mut HashMap<u64, Vec<String>>,
    map_crc : &mut HashMap<String, u32>,
) -> Result<(), io::Error> {
    let entries = fs::read_dir(dir)?;
    for entry in entries {
        let entry    = entry?;
        let metadata = entry.metadata()?;
        let path     = entry.path().display().to_string();
        if metadata.is_dir() {
            let _ = walk(path.as_str(), map_len, map_crc);
        } else {
            let lc_path = path.to_lowercase();
            if lc_path.ends_with(".cab") {
                let _ = archiver::cabinet::walk(path.as_str(), map_len, map_crc);
            } else if lc_path.ends_with(".zip") {
                let _ = archiver::zip::walk(path.as_str(), map_len, map_crc);
            } else {
                let len = metadata.len();
                if let Some(paths) = map_len.get_mut(&len) {
                    paths.push(path);
                } else {
                    map_len.insert(len, vec![path]);
                }
            }
        }
    }
    Ok(())
}

pub fn crc(path : &str) -> Result<u32, io::Error> {
    let mut crc: Option<u32> = None;
    let parts: Vec<&str> = path.split("\t").collect();
    if parts.len() == 1 {
        let mut hasher = Hasher::new();
        let file       = fs::File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut buffer = [0; 4096];
        loop {
            match reader.read(&mut buffer)? {
                0 => break,
                n => {
                    hasher.update(&buffer[..n]);
                }
            }
        }
        crc = Some(hasher.finalize());
    } else {
        let container = parts[0];
        let path      = parts[1];
        let lc_path   = container.to_lowercase();
        if lc_path.ends_with(".cab") {
            crc = Some(archiver::cabinet::crc(container, path)?);
        } else if lc_path.ends_with(".zip") {
            crc = Some(archiver::zip::crc(container, path)?);
        }
    }
    Ok(crc.unwrap_or(0))
}

pub fn remove(path : &str) -> Result<(), Box<dyn Error>>
{
    let parts: Vec<&str> = path.split("\t").collect();
    if parts.len() == 1 {
    } else {
        let container = parts[0];
        let path      = parts[1];
        let lc_path   = container.to_lowercase();
        if lc_path.ends_with(".cab") {
            archiver::cabinet::remove(container, vec![path.to_string()])?;
        } else if lc_path.ends_with(".zip") {
            archiver::zip::remove(container, vec![path.to_string()])?;
        }
    }
    Ok(())
}

pub fn remove_in_archive(container: &str, files: Vec<String>) -> Result<(), Box<dyn Error>>
{
    let lc_path = container.to_lowercase();
    if lc_path.ends_with(".cab") {
        archiver::cabinet::remove(container, files)?;
    } else if lc_path.ends_with(".zip") {
        archiver::zip::remove(container, files)?;
    }
    Ok(())
}
