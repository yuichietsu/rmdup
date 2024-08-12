use zip;
use std::error::Error;
use std::fs;
use std::io;
use std::collections::HashMap;
use crate::archiver;
use zip::read::ZipArchive;
use zip::write::FileOptions;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use chrono::prelude::*;

pub fn walk(
    file_name : &str,
    map_len   : &mut HashMap<u64, Vec<String>>,
    map_crc   : &mut HashMap<String, u32>,
) -> Result<(), io::Error> {
    let zip_file = fs::File::open(file_name)?;
    let reader   = io::BufReader::new(zip_file);
    let mut archive  = zip::ZipArchive::new(reader)?;
    for i in 0..archive.len() {
        let file = archive.by_index(i).unwrap();
        if file.is_file() {
            let len  = file.size();
            let name = format!("{}\t{}", file_name, file.name());
            archiver::push_map_len(map_len, len, name.as_str());
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

pub fn remove(container : &str, files : Vec<String>) -> Result<(), Box<dyn Error>> {
    let zip_file = File::open(container)?;
    let mut zip_archive = ZipArchive::new(zip_file)?;

    let now = Local::now();
    let now_date = now.format("%Y%m%d").to_string();
    let mut now_time = now.format("%H%M%S").to_string();

	let mut tmp_file;
	loop {
		tmp_file = format!("{}.{}_{}", container, now_date, now_time); 
		let tmp_path = Path::new(&tmp_file);
		if !tmp_path.exists() {
			break;
		}
		now_time.push_str("_");
	}

	let mut is_empty = true;
    let output_file = File::create(&tmp_file)?;
    let mut zip_writer = zip::ZipWriter::new(output_file);

    for i in 0..zip_archive.len() {
        let mut file = zip_archive.by_index(i)?;
        let file_name = file.name().to_string();

        if files.contains(&file_name) {
			println!("  Removed {}", file_name);
            continue;
        }

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        zip_writer.start_file(file_name, FileOptions::default())?;
        zip_writer.write_all(&buffer)?;

		is_empty = false;
    }
	zip_writer.finish()?;
	if is_empty {
		fs::remove_file(&tmp_file)?;
        println!("  Removed empty zip");
	} else {
        println!(
            "  {} => {} B",
            Path::new(container).metadata().unwrap().len(),
            Path::new(&tmp_file).metadata().unwrap().len()
         );
	}

    let _ = archiver::backup_archive(container, &now_date, &now_time);
    Ok(())
}
