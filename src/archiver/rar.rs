use std::error::Error;
use std::collections::HashMap;
use crate::archiver;
use tempfile::tempdir;
use std::fs;
use std::path::Path;
use std::process::Command;
use regex::Regex;
use std::io::{BufReader, Read};
use crc32fast::Hasher;

pub fn read_rar<F>(file_name: &str, mut callback: F) -> Result<(), Box<dyn Error>>
where F: FnMut(&str, bool, u64, u32)
{
    let output = Command::new("rar")
        .arg("lt")
        .arg(file_name)
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut name = "";
    let mut is_file = false;
    let mut size: u64 = 0;
    let mut crc: u32 = 0;
    for line in stdout.lines() {
        let line = line.trim();
        if line.starts_with("Name:") {
            name = &line[6..];
        } else if line.starts_with("Type:") {
            is_file = &line[6..] == "File"; 
        } else if line.starts_with("Size:") {
            size = (&line[6..]).parse()?;
        } else if line.starts_with("CRC32:") {
            crc = u32::from_str_radix(&line[7..], 16)?;
        } else if line == "" && name != "" {
            callback(name, is_file, size, crc);
            name = "";
            is_file = false;
            size = 0;
            crc = 0;
        }
    }
    Ok(())
}

pub fn walk(
    file_name : &str,
    map_len   : &mut HashMap<u64, Vec<String>>,
    map_crc   : &mut HashMap<String, u32>,
) -> Result<(), Box<dyn Error>> {
    let check_file = |name: &str, is_file: bool, size: u64, crc: u32| {
        if is_file {
            let name = format!("{}\t{}", file_name, name);
            archiver::push_map_len(map_len, size, name.as_str());
            if crc != 0 {
                map_crc.insert(name, crc); 
            }
        }
    };
    read_rar(file_name, check_file)?;
    Ok(())
}

pub fn crc(container : &str, path : &str) -> Result<u32, Box<dyn Error>> {
    let mut file_crc = 0;
    let check_file = |name: &str, is_file: bool, size: u64, crc: u32| {
        if is_file && name == path {
            if crc == 0 && size != 0 {
                let temp_dir = tempdir().unwrap();
                let t = temp_dir.path().to_str().unwrap();
                Command::new("rar")
                    .current_dir(t)
                    .arg("e")
                    .arg(container)
                    .arg(name)
                    .output()
                    .expect("Failed to execute command");
                let entries = fs::read_dir(t).unwrap();
                for entry in entries {
                    let entry    = entry.unwrap();
                    let mut hasher = Hasher::new();
                    let file       = fs::File::open(entry.path()).unwrap();
                    let mut reader = BufReader::new(file);
                    let mut buffer = [0; 4096];
                    loop {
                        match reader.read(&mut buffer).unwrap() {
                            0 => break,
                            n => {
                                hasher.update(&buffer[..n]);
                            }
                        }
                    }
                    file_crc = hasher.finalize();
                    break;
                }
            } else {
                file_crc = crc;
            }
        }
    };
    read_rar(container, check_file)?;
    Ok(file_crc)
}

pub fn remove(container : &str, files : Vec<String>) -> Result<(), Box<dyn Error>> {
    let temp_dir = tempdir()?;
	let t = temp_dir.path().to_str().unwrap();
	let mut is_empty = true;

    let check_file = |name: &str, is_file: bool, _size: u64, _crc: u32| {
        if is_file && !files.contains(&name.to_string()) {
            is_empty = false;
            Command::new("rar")
                .current_dir(t)
                .arg("x")
                .arg(container)
                .arg(name)
                .output()
                .expect("Failed to execute command");
        } else {
            println!("  Removed {}", name);
        }
    };
    read_rar(container, check_file)?;

    let now_str = archiver::now_str();

	if is_empty {
        println!("  Removed empty rar");
	} else {
        let tmp_file = archiver::resolve_tmp_path(&container, &now_str);

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

    archiver::backup_archive(container, &now_str)?;

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
                        archiver::backup_archive(path.to_str().unwrap(), &now_str)?;
                    }
                }
            }
        }
    }
    Ok(())
}

