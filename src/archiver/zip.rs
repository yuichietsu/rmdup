use zip;
use std::fs;
use std::io;
use std::collections::HashMap;

pub fn walk(
    file_name : &str,
    map_len   : &mut HashMap<u64, String>,
    _map_crc   : &mut HashMap<String, String>,
    map_cnt   : &mut HashMap<String, String>,
) -> Result<(), io::Error> {
    let zip_file = fs::File::open(file_name)?;
    let reader   = io::BufReader::new(zip_file);
    let mut archive  = zip::ZipArchive::new(reader).unwrap();
    for i in 0..archive.len() {
        let file = archive.by_index(i).unwrap();
        let len  = file.size();
        let name = file.name();
        map_len.insert(len, name.to_string()); 
        map_cnt.insert(name.to_string(), file_name.to_string());
    }
    Ok(())
}
