use std::error::Error;
use std::fs;
use std::io::{self, Read};
use std::collections::HashMap;
use cab;
use crc32fast::Hasher;
use crate::archiver;
use chrono::prelude::*;
use std::path::Path;

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
    let mut cab_builder = cab::CabinetBuilder::new();
    let new_folder = cab_builder.add_folder(cab::CompressionType::MsZip);

    let mut is_empty = true;
    let cabinet  = cab::Cabinet::new(fs::File::open(container)?)?;
    for folder in cabinet.folder_entries() {
        for file in folder.file_entries() {
            if file.name() != _path {
                is_empty = false;
                new_folder.add_file(file.name());        
            }
        }
    }

    let now = Local::now();
    let now_date = now.format("%Y%m%d").to_string();
    let mut now_time = now.format("%H%M%S").to_string();

    if is_empty {
        println!("{} : Removed", container);
    } else {
        let mut tmp_file;
        loop {
            tmp_file = format!("{}.{}_{}", container, now_date, now_time); 
            let tmp_path = Path::new(&tmp_file);
            if !tmp_path.exists() {
                break;
            }
            now_time.push_str("_");
        }
        let cab_file        = fs::File::create(tmp_file).unwrap();
        let mut cab_writer  = cab_builder.build(cab_file).unwrap();
        let mut cab_reader  = cab::Cabinet::new(fs::File::open(container)?)?;
        while let Some(mut writer) = cab_writer.next_file().unwrap() {
            let mut r = cab_reader.read_file(writer.file_name())?;
            io::copy(&mut r, &mut writer).unwrap();
        }
        let cab_file = cab_writer.finish().unwrap();
        println!(
            "{} : {} => {} B",
            container,
            Path::new(container).metadata().unwrap().len(),
            cab_file.metadata().unwrap().len()
         );
    }

    let _ = archiver::backup_archive(container, &now_date, &now_time);

    Ok(())
}
