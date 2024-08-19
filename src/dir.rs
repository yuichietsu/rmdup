use std::fs;
use std::error::Error;
use std::io;
use std::collections::HashMap;

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
            if !path.ends_with(".rmdup") {
                walk(path.as_str(), map_len, map_crc)?;
            }
        } else {
            let lc_path = path.to_lowercase();
            if lc_path.ends_with(".cab") {
                let _ = archiver::cabinet::walk(path.as_str(), map_len, map_crc);
            } else if lc_path.ends_with(".zip") {
                let _ = archiver::zip::walk(path.as_str(), map_len, map_crc);
            } else if lc_path.ends_with(".lzh") {
                let _ = archiver::lzh::walk(path.as_str(), map_len, map_crc);
            } else if lc_path.ends_with(".rar") {
                match archiver::rar::walk(path.as_str(), map_len, map_crc) {
                    Ok(()) => {
                    },
                    Err(e) => {
                        eprintln!("SKIP : {} : {}", path, e);
                    },
                }
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

pub fn crc(path : &str) -> Result<u32, Box<dyn Error>> {
    let mut crc: u32 = 0;
    let parts: Vec<&str> = path.split("\t").collect();
    if parts.len() == 1 {
        crc = archiver::make_crc_from_path(&path)?;
    } else {
        let container = parts[0];
        let path      = parts[1];
        let lc_path   = container.to_lowercase();
        if lc_path.ends_with(".cab") {
            crc = archiver::cabinet::crc(container, path)?;
        } else if lc_path.ends_with(".zip") {
            crc = archiver::zip::crc(container, path)?;
        } else if lc_path.ends_with(".lzh") {
            crc = archiver::lzh::crc(container, path)?;
        } else if lc_path.ends_with(".rar") {
            crc = archiver::rar::crc(container, path)?;
        }
    }
    Ok(crc)
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
    } else if lc_path.ends_with(".lzh") {
        archiver::lzh::remove(container, files)?;
    } else if lc_path.ends_with(".rar") {
        archiver::rar::remove(container, files)?;
    }
    Ok(())
}

pub fn backup_file(file: &str) -> Result<(), Box<dyn Error>>
{
    let now_str = archiver::now_str();
    archiver::backup_archive(file, &now_str)?;
    Ok(())
}
