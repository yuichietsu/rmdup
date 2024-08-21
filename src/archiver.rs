use std::fs;
use std::fs::File;
use std::error::Error;
use std::env;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use chrono::prelude::Local;
use cached::proc_macro::cached;
use regex;
use std::io::{BufReader, Read};
use crc32fast::Hasher;
use encoding_rs::SHIFT_JIS;

pub mod cabinet;
pub mod zip;
pub mod lzh;
pub mod rar;

pub fn push_map_len(map_len: &mut HashMap<u64, Vec<String>>, len: u64, name: &str)
{
    if let Some(paths) = map_len.get_mut(&len) {
        paths.push(name.to_string());
    } else {
        map_len.insert(len, vec![name.to_string()]);
    }
}

#[cached]
pub fn now_str() -> String
{
    let now = Local::now();
    now.format("%Y%m%d_%H%M%S").to_string()
}

pub fn resolve_tmp_path(path: &str, now_str: &str) -> String
{
    format!("{}.tmp.{}", path, now_str)
}

pub fn backup_archive(path: &str, now_str: &str) -> Result<(), Box<dyn Error>>
{
    let home_dir = match env::var("RMDUP_HOME") {
        Ok(val) => val.trim_end_matches('/').to_string(),
        Err(_)  => env::temp_dir().to_str().unwrap_or("").to_string(),
    };
    let mut pt = String::new();
    pt.push_str("^");
    pt.push_str(&regex::escape(&home_dir));
    let re = regex::Regex::new(&pt)?;
    let bak_path = format!(
        "{}/.rmdup/{}{}",
        home_dir,
        now_str,
        re.replace(path, "")
    );
	if let Some(parent) = PathBuf::from(&bak_path).parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    let tmp_path = resolve_tmp_path(path, now_str);

	fs::rename(path, &bak_path)?;
	let src = Path::new(&tmp_path);
	if src.exists() {
		fs::rename(&tmp_path, path)?;
	}
	Ok(())
}

pub fn make_crc_from_path(path: &str) -> Result<u32, Box<dyn Error>>
{
    let file       = fs::File::open(path)?;
    let mut reader = BufReader::new(file);
    make_crc_from_buf_reader(&mut reader)
}

pub fn make_crc_from_file(file: File) -> Result<u32, Box<dyn Error>>
{
    let mut reader = BufReader::new(file);
    make_crc_from_buf_reader(&mut reader)
}

pub fn make_crc_from_reader(reader: &mut dyn Read) -> Result<u32, Box<dyn Error>>
{
    let mut buf_reader = BufReader::new(reader);
    make_crc_from_buf_reader(&mut buf_reader)
}

pub fn make_crc_from_buf_reader<R: Read>(reader: &mut BufReader<R>) -> Result<u32, Box<dyn Error>>
{
    let mut hasher = Hasher::new();
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

pub fn to_utf8(bytes: &[u8]) -> String
{
	let file_name_utf8 = std::str::from_utf8(bytes);
	match file_name_utf8 {
		Ok(valid_str) => valid_str.to_string(),
		Err(_) => {
			let (decoded_str, _, _) = SHIFT_JIS.decode(bytes);
			decoded_str.to_string()
		}
	}
}
