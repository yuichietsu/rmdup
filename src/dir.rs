use std::fs;
use std::io;
use std::collections::HashMap;

use crate::archiver;

pub fn walk(
    dir     : &str,
    map_len : &mut HashMap<u64, String>,
    map_crc : &mut HashMap<String, String>,
    map_cnt : &mut HashMap<String, String>,
) -> Result<(), io::Error> {
    let entries = fs::read_dir(dir)?;
    for entry in entries {
        let entry    = entry?;
        let metadata = entry.metadata()?;
        let path     = entry.path().display().to_string();
        if metadata.is_dir() {
            let _ = walk(path.as_str(), map_len, map_crc, map_cnt);
        } else {
            let lc_path = path.to_lowercase();
            if lc_path.ends_with(".cab") {
                let _ = archiver::cabinet::walk(path.as_str(), map_len, map_crc, map_cnt);
            } else if lc_path.ends_with(".zip") {
                let _ = archiver::zip::walk(path.as_str(), map_len, map_crc, map_cnt);
            }
        }
    }
    Ok(())
}
