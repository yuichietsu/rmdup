use std::fs;
use std::io;
use std::env;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use chrono::prelude::Local;
use cached::proc_macro::cached;

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

pub fn backup_archive(path: &str, now_str: &str) -> Result<(), io::Error>
{
    let bak_path = format!("{}/rmdup/{}{}", env::temp_dir().display(), now_str, path);
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
