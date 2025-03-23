use clap::{App, Arg};
use std::collections::HashMap;
use std::error::Error;
use std::env;
use rmdup::dir;
use rmdup::file_info::{FileInfo, DuplicateGroup};
use rayon::prelude::*;

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

pub fn calc_ext_prior(path: &str) -> u8 {
    let path_lower = path.to_lowercase();
    match path_lower.as_str() {
        p if p.ends_with(".zip") => 8,
        p if p.ends_with(".cab") => 6,
        _ => 0,
    }
}
