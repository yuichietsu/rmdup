use clap::{App, Arg};
use std::collections::HashMap;
use std::error::Error;
use std::env;
use std::path::Path;
use rayon::prelude::*;

use rmdup::dir;

#[derive(Debug)]
struct FileInfo {
    container: Option<String>,
    path: String,
    crc: u32,
}

impl FileInfo {
    fn new(path: &str, _length: u64, crc: u32) -> Self {
        let parts: Vec<&str> = path.split('\t').collect();
        let (container, path) = if parts.len() == 2 {
            (Some(parts[0].to_string()), parts[1].to_string())
        } else {
            (None, parts[0].to_string())
        };
        Self { container, path, crc }
    }

    fn get_extension(&self) -> Option<String> {
        Path::new(&self.path)
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
    }
}

#[derive(Debug)]
struct DuplicateGroup {
    files: Vec<FileInfo>,
}

impl DuplicateGroup {
    fn new(_length: u64, _crc: u32, files: Vec<FileInfo>) -> Self {
        Self { files }
    }

    fn sort_files(&mut self) {
        self.files.sort_by(|a, b| {
            let a_priority = a.container.as_ref()
                .map(|c| calc_ext_prior(c))
                .unwrap_or(0);
            let b_priority = b.container.as_ref()
                .map(|c| calc_ext_prior(c))
                .unwrap_or(0);
            if a_priority == b_priority {
                a.path.cmp(&b.path)
            } else {
                b_priority.cmp(&a_priority)
            }
        });
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("remove duplicated files")
        .version("0.0.1")
        .version_message("show version")
        .help_message("show help")
        .arg(Arg::with_name("dir").help("scan directory"))
        .arg(Arg::with_name("move").long("move").takes_value(true).help("move if ext included in archive"))
        .get_matches();

    let mut map_len = HashMap::new();
    let mut map_crc = HashMap::new();

    if let Some(dir) = matches.value_of("dir") {
        env::set_var("RMDUP_SCAN_DIR", dir);
        dir::walk(dir, &mut map_len, &mut map_crc)?;

        let move_ext = matches.value_of("move").unwrap_or("");
        if move_ext.is_empty() {
            remove_duplicated_files(&mut map_len, &mut map_crc)?;
        } else {
            move_if_ext_included(move_ext, &mut map_len)?;
        }
    } else {
        println!("scan directory not specified.");
    }
    Ok(())
}

fn move_if_ext_included(
    move_ext: &str,
    map_len: &mut HashMap<u64, Vec<String>>
) -> Result<(), Box<dyn Error>> {
    let mut removed_containers: Vec<String> = Vec::new();
    for (_len, paths) in map_len {
        for path in paths.iter() {
            let file_info = FileInfo::new(path, 0, 0);
            if let Some(container) = &file_info.container {
                if !removed_containers.contains(container) {
                    if let Some(ext) = file_info.get_extension() {
                        if ext == move_ext {
                            println!("[MOVE={}]", container);
                            dir::backup_file(container)?;
                            removed_containers.push(container.clone());
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn remove_duplicated_files(
    map_len: &mut HashMap<u64, Vec<String>>,
    map_crc: &mut HashMap<String, u32>,
) -> Result<(), Box<dyn Error>> {
    let mut rlist: HashMap<String, Vec<String>> = HashMap::new();
    
    for (len, paths) in map_len {
        if paths.len() <= 1 {
            continue;
        }
        println!("[LEN={}, COUNT={}]", len, paths.len());

        let results: Vec<_> = paths.par_iter()
            .map(|path| {
                let crc = map_crc.get(path)
                    .copied()
                    .unwrap_or_else(|| dir::crc(path).unwrap());
                FileInfo::new(path, 0, crc)
            })
            .collect();

        let mut map_dup: HashMap<u32, Vec<FileInfo>> = HashMap::new();
        for file_info in results {
            map_dup.entry(file_info.crc)
                .or_insert_with(Vec::new)
                .push(file_info);
        }

        for (crc, files) in map_dup {
            if files.len() > 1 {
                let mut group = DuplicateGroup::new(0, crc, files);
                group.sort_files();
                
                println!("[LEN={}, CRC={}]", len, crc);
                let keep_file = group.files.pop().unwrap();
                println!("*** {}", keep_file.path);
                
                for file in group.files {
                    if let Some(container) = file.container {
                        let path = file.path.clone();
                        rlist.entry(container)
                            .or_insert_with(Vec::new)
                            .push(path);
                        println!("--- {}", file.path);
                    } else {
                        println!("  Removed {}", file.path);
                        dir::backup_file(&file.path)?;
                    }
                }
            }
        }
    }
    
    for (container, files) in rlist {
        println!("[ARC={}]", container);
        dir::remove_in_archive(&container, files)?;
    }
    Ok(())
}

fn calc_ext_prior(path: &str) -> u8 {
    let path_lower = path.to_lowercase();
    match path_lower.as_str() {
        p if p.ends_with(".zip") => 8,
        p if p.ends_with(".cab") => 6,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_info_creation_normal_path() {
        let file_info = FileInfo::new("test.txt", 0, 12345);
        assert_eq!(file_info.path, "test.txt");
        assert_eq!(file_info.container, None);
        assert_eq!(file_info.crc, 12345);
    }

    #[test]
    fn test_file_info_creation_with_container() {
        let file_info = FileInfo::new("archive.zip\ttest.txt", 0, 12345);
        assert_eq!(file_info.path, "test.txt");
        assert_eq!(file_info.container, Some("archive.zip".to_string()));
        assert_eq!(file_info.crc, 12345);
    }

    #[test]
    fn test_file_info_get_extension() {
        let file_info = FileInfo::new("test.TXT", 0, 12345);
        assert_eq!(file_info.get_extension(), Some("txt".to_string()));

        let file_info = FileInfo::new("test", 0, 12345);
        assert_eq!(file_info.get_extension(), None);
    }

    #[test]
    fn test_duplicate_group_sorting() {
        let files = vec![
            FileInfo::new("test.txt", 0, 1),
            FileInfo::new("archive.zip\tfile.txt", 0, 2),
            FileInfo::new("data.cab\tfile.txt", 0, 3),
            FileInfo::new("other.txt", 0, 4),
        ];

        let mut group = DuplicateGroup::new(0, 0, files);
        group.sort_files();

        // ZIPが最優先、次にCAB、最後に通常ファイル
        assert_eq!(group.files[0].container, Some("archive.zip".to_string()));
        assert_eq!(group.files[1].container, Some("data.cab".to_string()));
        assert!(group.files[2].container.is_none());
        assert!(group.files[3].container.is_none());
    }

    #[test]
    fn test_calc_ext_prior() {
        assert_eq!(calc_ext_prior("test.zip"), 8);
        assert_eq!(calc_ext_prior("test.ZIP"), 8);
        assert_eq!(calc_ext_prior("test.Zip"), 8);
        assert_eq!(calc_ext_prior("test.cab"), 6);
        assert_eq!(calc_ext_prior("test.CAB"), 6);
        assert_eq!(calc_ext_prior("test.Cab"), 6);
        assert_eq!(calc_ext_prior("test.txt"), 0);
        assert_eq!(calc_ext_prior("test"), 0);
    }
}
