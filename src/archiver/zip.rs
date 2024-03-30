use zip;
use std::fs;
use std::io;
use std::collections::HashMap;

pub fn walk(
    file_name : &str,
    map_len   : &mut HashMap<u64, Vec<String>>,
    _map_crc   : &mut HashMap<String, String>,
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
        }
    }
    Ok(())
}
