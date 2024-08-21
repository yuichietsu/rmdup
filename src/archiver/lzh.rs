use std::error::Error;
use std::io;
use std::collections::HashMap;
use crate::archiver;
use delharc;
use tempfile::tempdir;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn walk(
    file_name : &str,
    map_len   : &mut HashMap<u64, Vec<String>>,
    _map_crc   : &mut HashMap<String, u32>,
) -> Result<(), Box<dyn Error>> {
    let mut lha_reader = delharc::parse_file(file_name)?;
    loop {
        let header = lha_reader.header();
        let filename = archiver::to_utf8(&header.filename);

		if lha_reader.is_decoder_supported() {
			let len = header.original_size;
            let name = format!("{}\t{}", file_name, filename);
            archiver::push_map_len(map_len, len, &name);
		}

        if !lha_reader.next_file()? {
            break;
        }
    }
    Ok(())
}

pub fn crc(container : &str, path : &str) -> Result<u32, Box<dyn Error>> {
    let mut lha_reader = delharc::parse_file(container)?;
    loop {
        let header = lha_reader.header();
        let filename = archiver::to_utf8(&header.filename);
        if filename.ends_with(path) {
            return Ok(archiver::make_crc_from_reader(&mut lha_reader)?);
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
        let filename = archiver::to_utf8(&header.filename);
		let p = format!("{}/{}", t, filename);
        if files.contains(&filename) {
			println!("  Removed {}", filename);
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
			} else {
                println!("  Skipped {}", filename);
            }
        }
        if !lha_reader.next_file()? {
            break;
        }
    }

    let now_str = archiver::now_str();

	if is_empty {
        println!("  Removed empty lzh");
	} else {
        let tmp_file = archiver::resolve_tmp_path(&container, &now_str);

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

    archiver::backup_archive(container, &now_str)?;
    Ok(())
}
