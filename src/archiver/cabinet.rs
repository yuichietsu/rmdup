use std::error::Error;
use std::fs;
use std::io;
use std::collections::HashMap;
use cab;
use crate::archiver;
use std::path::Path;

pub fn walk(
    file_name : &str,
    map_len   : &mut HashMap<u64, Vec<String>>,
    _map_crc   : &mut HashMap<String, u32>,
) -> Result<(), Box<dyn Error>> {
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

pub fn crc(container : &str, path : &str) -> Result<u32, Box<dyn Error>> {
    let cab_file = fs::File::open(container)?;
    let mut cabinet = cab::Cabinet::new(cab_file)?;
    let mut reader = cabinet.read_file(path)?;
    Ok(archiver::make_crc_from_reader(&mut reader)?)
}

pub fn remove(container : &str, files : Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut cab_builder = cab::CabinetBuilder::new();
    let new_folder = cab_builder.add_folder(cab::CompressionType::MsZip);

    let mut is_empty = true;
    let cabinet  = cab::Cabinet::new(fs::File::open(container)?)?;
    for folder in cabinet.folder_entries() {
        for file in folder.file_entries() {
            if files.contains(&file.name().to_string()) {
                println!("  Removed {}", file.name());
            } else {
                is_empty = false;
                new_folder.add_file(file.name());        
            }
        }
    }

    let now_str = archiver::now_str();

    if is_empty {
        println!("  Removed empty cabinet");
    } else {
        let tmp_file       = archiver::resolve_tmp_path(&container, &now_str);
        let cab_file       = fs::File::create(&tmp_file).unwrap();
        let mut cab_writer = cab_builder.build(cab_file).unwrap();
        let mut cab_reader = cab::Cabinet::new(fs::File::open(container)?)?;
        while let Some(mut writer) = cab_writer.next_file().unwrap() {
            let mut r = cab_reader.read_file(writer.file_name())?;
            io::copy(&mut r, &mut writer).unwrap();
        }
        let cab_file = cab_writer.finish().unwrap();
        println!(
            "  {} => {} B",
            Path::new(container).metadata().unwrap().len(),
            cab_file.metadata().unwrap().len()
         );
    }

    archiver::backup_archive(container, &now_str)?;
    Ok(())
}
