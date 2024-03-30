use cab;
use std::fs;
use std::io;
use std::collections::HashMap;

pub fn walk(
    file_name : &str,
    map_len   : &mut HashMap<u64, String>,
    _map_crc   : &mut HashMap<String, String>,
    map_cnt   : &mut HashMap<String, String>,
) -> Result<(), io::Error> {
    let cab_file = fs::File::open(file_name)?;
    let cabinet = cab::Cabinet::new(cab_file)?;
    for folder in cabinet.folder_entries() {
        for file in folder.file_entries() {
            let len  = file.uncompressed_size();
            let name = file.name();
            map_len.insert(len as u64, name.to_string()); 
            map_cnt.insert(name.to_string(), file_name.to_string());
        }
    }
    Ok(())
}
