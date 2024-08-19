use clap::{App, Arg};
use std::collections::HashMap;
use std::error::Error;
use std::env;
use std::path::Path;

use rmdup::dir;

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
        env::set_var("RMDUP_HOME", dir);
        dir::walk(dir, &mut map_len, &mut map_crc)?;

        let move_ext = matches.value_of("move").unwrap_or("");
        if move_ext == "" {
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
    move_ext : &str,
    map_len : &mut HashMap<u64, Vec<String>>
) -> Result<(), Box<dyn Error>> {
    let mut removed_containers: Vec<&str> = Vec::new();
    for (_len, paths) in map_len {
        for path in paths.iter() {
            let dat: Vec<&str> = path.split("\t").collect();
            if dat.len() == 2 {
                let c = dat[0];
                if !removed_containers.contains(&c) {
                    let p = Path::new(dat[1]);
                    if let Some(p_ext) = p.extension() {
                        if let Some(p_ext_str) = p_ext.to_str() {
                            let lp_ext_str = p_ext_str.to_lowercase();
                            if lp_ext_str == move_ext {
                                dir::backup_file(c)?;
                                removed_containers.push(c);
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn remove_duplicated_files(
    map_len : &mut HashMap<u64, Vec<String>>,
    map_crc : &mut HashMap<String, u32>,
) -> Result<(), Box<dyn Error>> {
    let mut rlist: HashMap<String, Vec<String>> = HashMap::new();
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
                let mut sorted = dups.clone();
                sorted.sort_by(|a, b| {
                    let ap: Vec<&str> = a.split("\t").collect();
                    let bp: Vec<&str> = b.split("\t").collect();
                    let apl = ap.len();
                    let bpl = bp.len();
                    if apl == bpl {
                        let ac = ap[0];
                        let bc = bp[0];
                        let acp = calc_ext_prior(ac);
                        let bcp = calc_ext_prior(bc);
                        if acp == bcp {
                            ac.cmp(bc)
                        } else {
                            acp.cmp(&bcp)
                        }
                    } else {
                        apl.cmp(&bpl)
                    }
                });
                println!("[LEN={}, CRC={}]", len, crc);
                let path = sorted.pop().unwrap();
                println!("*** {}",path);
                for path in sorted.iter() {
                    let sp: Vec<&str> = path.split("\t").collect();
                    if sp.len() == 1 {
                        println!("  Removed {}", path);
                        dir::backup_file(path)?;
                    } else {
                        let c = sp[0];
                        let f = sp[1];
                        if let Some(container) = rlist.get_mut(c) {
                            container.push(f.to_string());
                        } else {
                            rlist.insert(c.to_string(), vec![f.to_string()]);
                        }
                        println!("--- {}", path);
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

fn push_key_values(map_dup: &mut HashMap<u32, Vec<String>>, crc: u32, path: &str)
{
    if let Some(dups) = map_dup.get_mut(&crc) {
        dups.push(path.to_string());
    } else {
        map_dup.insert(crc, vec![path.to_string()]);
    }
}

fn calc_ext_prior(path: &str) -> u8
{
    match path.to_lowercase() {
        p if p.ends_with(".zip") => 8,
        p if p.ends_with(".cab") => 6,
        _ => 0,
    }
}
