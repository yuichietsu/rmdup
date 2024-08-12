use std::fs;
use std::io;
use std::env;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub mod cabinet;
pub mod zip;

pub fn push_map_len(map_len: &mut HashMap<u64, Vec<String>>, len: u64, name: &str)
{
    if let Some(paths) = map_len.get_mut(&len) {
        paths.push(name.to_string());
    } else {
        map_len.insert(len, vec![name.to_string()]);
    }
}

pub fn backup_archive(path: &str, now_date: &str, now_time: &str) -> Result<(), io::Error>
{
    let bk_path = format!("{}/rmdup/{}{}", env::temp_dir().display(), now_date, path);
	let mut bk_path = PathBuf::from(bk_path);
	let dst_file = format!("{}.{}", now_time, bk_path.file_name().unwrap().to_string_lossy());
	bk_path.set_file_name(dst_file);

    let new_path = format!("{}.{}_{}", path, now_date, now_time);
    
	if let Some(parent) = bk_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

	let bk_path = bk_path.display().to_string();
	fs::rename(path, &bk_path)?;
	let src = Path::new(&new_path);
	if src.exists() {
		fs::rename(&new_path, path)?;
	}
	Ok(())
}
