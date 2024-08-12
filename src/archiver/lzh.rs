use std::error::Error;
use std::io::{self, Read};
use std::collections::HashMap;
use crate::archiver;
use delharc;
use crc32fast::Hasher;

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
		else if header.is_directory() {
			eprintln!("skipping: an empty directory");
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
	for file in files {
		println!("  {} in {}", file, container);
	}
    Ok(())
}
