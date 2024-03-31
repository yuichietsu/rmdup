use zip;
use std::fs;
use std::io;
use std::collections::HashMap;

pub fn walk(
    file_name : &str,
    map_len   : &mut HashMap<u64, Vec<String>>,
    map_crc   : &mut HashMap<String, u32>,
) -> Result<(), io::Error> {
    let zip_file = fs::File::open(file_name)?;
    let reader   = io::BufReader::new(zip_file);
    let mut archive  = zip::ZipArchive::new(reader).unwrap();
    for i in 0..archive.len() {
        let file = archive.by_index(i).unwrap();
        if file.is_file() {
            let len  = file.size();
            let name = format!("{}\t{}", file_name, file.name());
            if let Some(paths) = map_len.get_mut(&len) {
                paths.push(name);
            } else {
                map_len.insert(len, vec![name]);
            }
            let name = format!("{}\t{}", file_name, file.name());
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
