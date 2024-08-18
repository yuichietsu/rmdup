use std::error::Error;
use std::io;
use std::collections::HashMap;
use crate::archiver;
use tempfile::tempdir;
use std::fs;
use chrono::prelude::*;
use std::path::Path;
use std::process::Command;
use unrar::Archive;
use regex::Regex;

pub fn walk(
    file_name : &str,
    map_len   : &mut HashMap<u64, Vec<String>>,
    map_crc   : &mut HashMap<String, u32>,
) -> Result<(), io::Error> {
    for entry in Archive::new(file_name).open_for_listing().unwrap() {
        let e = entry.unwrap();
        if !e.is_directory() {
            let len  = e.unpacked_size;
            let name = format!("{}\t{}", file_name, e.filename.display());
            archiver::push_map_len(map_len, len, name.as_str());
            map_crc.insert(name, e.file_crc); 
        }
    }
    Ok(())
}

pub fn crc(container : &str, path : &str) -> Result<u32, io::Error> {
    for entry in Archive::new(container).open_for_listing().unwrap() {
        let e = entry.unwrap();
        if !e.is_directory() && e.filename.as_path().ends_with(path) {
            return Ok(e.file_crc)
        }
    }
    Ok(0)
}

pub fn remove(container : &str, files : Vec<String>) -> Result<(), Box<dyn Error>> {
    let temp_dir = tempdir()?;
	let t = temp_dir.path().to_str().unwrap();
	let mut is_empty = true;

    let mut archive = Archive::new(container).open_for_processing().unwrap();
    while let Some(header) = archive.read_header()? {
		let filename = header.entry().filename.display().to_string();
		let p = format!("{}/{}", t, filename);
        archive = if !files.contains(&filename) {
			is_empty = false;
			let path = Path::new(&p);
			if let Some(parent) = path.parent() {
				if !parent.exists() {
					fs::create_dir_all(parent)?;
				}
			}
			header.extract_to(p)?
        } else {
			println!("  Removed {}", p);
            header.skip()?
        }
    }

	let now = Local::now();
	let now_date = now.format("%Y%m%d").to_string();
	let mut now_time = now.format("%H%M%S").to_string();

	if is_empty {
        println!("  Removed empty rar");
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

		let output = Command::new("rar")
			.current_dir(t)
			.arg("a")
            .arg("-r")
			.arg(&tmp_file)
			.arg(".")
			.output()
			.expect("Failed to execute command");

		if !output.status.success() {
			eprintln!("Error: {:?}", output);
		}

        println!(
            "  {} => {} B",
            Path::new(container).metadata().unwrap().len(),
            Path::new(&tmp_file).metadata().unwrap().len()
         );
	}

    let _ = archiver::backup_archive(container, &now_date, &now_time);

    let re = Regex::new(r"^r\d{2}$").unwrap();
    let path = Path::new(container);
    let stem = path.file_stem();
    if let Some(parent) = path.parent() {
        let entries = fs::read_dir(parent)?;
        for entry in entries {
            let path = entry?.path();
            if stem == path.file_stem() {
                if let Some(ext) = path.extension() {
                    if re.is_match(ext.to_str().unwrap()) {
                        let _ = archiver::backup_archive(path.to_str().unwrap(), &now_date, &now_time);
                    }
                }
            }
        }
    }
    Ok(())
}

