use std::fs;
use std::error::Error;
use std::collections::HashMap;

use crate::archiver;

pub fn walk(
    dir     : &str,
    map_len : &mut HashMap<u64, Vec<String>>,
    map_crc : &mut HashMap<String, u32>,
) -> Result<(), Box<dyn Error>> {
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
            let r;
            let lc_path = path.to_lowercase();
            if lc_path.ends_with(".cab") {
                r = archiver::cabinet::walk(path.as_str(), map_len, map_crc);
            } else if lc_path.ends_with(".zip") {
                r = archiver::zip::walk(path.as_str(), map_len, map_crc);
            } else if lc_path.ends_with(".lzh") {
                r = archiver::lzh::walk(path.as_str(), map_len, map_crc);
            } else if lc_path.ends_with(".rar") {
                r = archiver::rar::walk(path.as_str(), map_len, map_crc);
            } else {
                r = Ok(());
                let len = metadata.len();
                if let Some(paths) = map_len.get_mut(&len) {
                    paths.push(path);
                } else {
                    map_len.insert(len, vec![path]);
                }
            }
            match r {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("Skip {} : Error = {:?}", entry.path().display(), e);
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
