use clap::{App, Arg};
use std::collections::HashMap;
use std::error::Error;

use rmdup::dir;

fn main() -> Result<(), Box<dyn Error>> {
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
                        println!("--- {}", path);
                        dir::remove(path)?;
                    }
                }
            }
        }
    } else {
        println!("scan directory not specified.");
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
