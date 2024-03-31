use clap::{App, Arg};
use std::collections::HashMap;
use std::io;

use rmdup::dir;

fn main() -> Result<(), io::Error> {
    let matches = App::new("remove duplicated files")
        .version("0.0.1")
        .version_message("show version")
        .help_message("show help")
        .arg(Arg::with_name("dir").help("scan directory"))
        .get_matches();

    let mut map_len = HashMap::new();
    let mut map_crc = HashMap::new();

    if let Some(dir) = matches.value_of("dir") {
        let _ = dir::walk(dir, &mut map_len, &mut map_crc);
        for (len, paths) in map_len {
            let mut map_dup: HashMap<u32, Vec<String>> = HashMap::new();
            for path in paths.iter() {
                if let Some(crc) = map_crc.get(path) {
                    push_key_values(&mut map_dup, *crc, path);
                } else {
                    let crc = dir::crc(path)?;
                    push_key_values(&mut map_dup, crc, path);
                }
            }
            for (crc, dups) in map_dup {
                if dups.len() > 1 {
                    for path in dups.iter() {
                        println!("{} : {} : {}", len, crc, path);
                    }
                }
            }
        }
    } else {
        println!("scan directory not secified.");
    }
    Ok(())
}

fn push_key_values(map_dup: &mut HashMap<u32, Vec<String>>, crc: u32, path: &str)
{
    if let Some(dups) = map_dup.get_mut(&crc) {
        dups.push(path.to_string());
    } else {
        map_dup.insert(crc, vec![path.to_string()]);
    }
}
