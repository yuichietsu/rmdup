use std::error::Error;
use std::io::{self, Read};
use std::collections::HashMap;
use crate::archiver;
use delharc;
use crc32fast::Hasher;
use tempfile::tempdir;
use std::fs;
use chrono::prelude::*;
use std::path::Path;
use std::process::Command;

pub fn walk(
    file_name : &str,
    map_len   : &mut HashMap<u64, Vec<String>>,
    _map_crc   : &mut HashMap<String, u32>,
) -> Result<(), io::Error> {
    let mut lha_reader = delharc::parse_file(file_name)?;
    loop {
        let header = lha_reader.header();
        let filename = header.parse_pathname().display().to_string();

		if lha_reader.is_decoder_supported() {
			let len = header.original_size;
            let name = format!("{}\t{}", file_name, filename);
            archiver::push_map_len(map_len, len, &name);
		}
		else {
			eprintln!("skipping: has unsupported compression method");
		}

        if !lha_reader.next_file()? {
            break;
        }
    }
    Ok(())
}

pub fn crc(container : &str, path : &str) -> Result<u32, io::Error> {
    let mut hasher = Hasher::new();
    let mut lha_reader = delharc::parse_file(container)?;
    loop {
        let header = lha_reader.header();
        let filename = header.parse_pathname();
        if filename.ends_with(path) {
            let mut buffer = [0; 4096];
            loop {
                match lha_reader.read(&mut buffer)? {
                    0 => break,
                    n => {
                        hasher.update(&buffer[..n]);
                    }
                }
            }
            return Ok(hasher.finalize())
        }
        if !lha_reader.next_file()? {
            break;
        }
    }
    Ok(0)
}

pub fn remove(container : &str, files : Vec<String>) -> Result<(), Box<dyn Error>> {
    let temp_dir = tempdir()?;
    let mut lha_reader = delharc::parse_file(container)?;
	let t = temp_dir.path().to_str().unwrap();
	let mut is_empty = true;
    loop {
        let header = lha_reader.header();
        let filename = header.parse_pathname().display().to_string();
		let p = format!("{}/{}", t, filename);
        if files.contains(&filename) {
			println!("  Removed {}", p);
		} else {
			if lha_reader.is_decoder_supported() {
				let path = Path::new(&p);
				if let Some(parent) = path.parent() {
					if !parent.exists() {
						fs::create_dir_all(parent)?;
					}
				}
				let mut w = fs::File::create(&p)?;
				io::copy(&mut lha_reader, &mut w)?;
				is_empty = false;
			}
        }
        if !lha_reader.next_file()? {
            break;
        }
    }

	let now = Local::now();
	let now_date = now.format("%Y%m%d").to_string();
	let mut now_time = now.format("%H%M%S").to_string();

	if is_empty {
        println!("  Removed empty lzh");
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

		let output = Command::new("jlha")
			.current_dir(t)
			.arg("a")
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
    Ok(())
}
