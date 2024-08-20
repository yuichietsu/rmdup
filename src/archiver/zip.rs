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
use zip::read::ZipFile;
use encoding_rs::SHIFT_JIS;

pub fn walk(
    container : &str,
    map_len   : &mut HashMap<u64, Vec<String>>,
    map_crc   : &mut HashMap<String, u32>,
) -> Result<(), Box<dyn Error>> {
    let zip_file = fs::File::open(container)?;
    let reader   = io::BufReader::new(zip_file);
    let mut archive  = zip::ZipArchive::new(reader)?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        if file.is_file() {
            let len  = file.size();
            let name = format!("{}\t{}", container, normalize_file_name(&mut file));
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

    let now_str  = archiver::now_str();
    let tmp_file = archiver::resolve_tmp_path(&container, &now_str);

	let mut is_empty = true;
    let output_file = File::create(&tmp_file)?;
    let mut zip_writer = zip::ZipWriter::new(output_file);

    for i in 0..zip_archive.len() {
        let mut file = zip_archive.by_index(i)?;
        let file_name = normalize_file_name(&mut file);

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

    archiver::backup_archive(container, &now_str)?;
    Ok(())
}

pub fn normalize_file_name(file: &mut ZipFile) -> String
{
	let file_name_bytes = file.name_raw();
	let file_name_utf8 = std::str::from_utf8(file_name_bytes);
	match file_name_utf8 {
		Ok(valid_str) => valid_str.to_string(),
		Err(_) => {
			let (decoded_str, _, _) = SHIFT_JIS.decode(file_name_bytes);
			decoded_str.to_string()
		}
	}
}
