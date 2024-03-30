use cab;
use std::fs;
use std::io;
use std::collections::HashMap;

pub fn walk(
    file_name : &str,
    map_len   : &mut HashMap<u64, Vec<String>>,
    _map_crc   : &mut HashMap<String, String>,
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
