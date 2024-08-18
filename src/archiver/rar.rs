use std::error::Error;
use std::io;
use std::collections::HashMap;
use crate::archiver;
use tempfile::tempdir;
use std::fs;
use std::path::Path;
use std::process::Command;
use unrar::Archive;
use regex::Regex;

pub fn walk(
    file_name : &str,
    map_len   : &mut HashMap<u64, Vec<String>>,
    map_crc   : &mut HashMap<String, u32>,
) -> Result<(), Box<dyn Error>> {
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
            if is_file && crc != 0 {
                let name = format!("{}\t{}", file_name, name);
                archiver::push_map_len(map_len, size, name.as_str());
                map_crc.insert(name, crc); 
            }
            name = "";
            is_file = false;
            size = 0;
            crc = 0;
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

    let output = Command::new("rar")
        .arg("lt")
        .arg(container)
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut name = "";
    let mut is_file = false;
    for line in stdout.lines() {
        let line = line.trim();
        if line.starts_with("Name:") {
            name = &line[6..];
        } else if line.starts_with("Type:") {
            is_file = &line[6..] == "File"; 
        } else if line == "" && name != "" {
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
            name = "";
            is_file = false;
        }
    }

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

