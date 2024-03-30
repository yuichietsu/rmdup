use clap::{App, Arg};
use std::collections::HashMap;

mod dir;

fn main() {
    let matches = App::new("remove duplicated files")
        .version("0.0.1")
        .version_message("show version")
        .help_message("show help")
        .arg(Arg::with_name("dir").help("scan directory"))
        .get_matches();

    let mut files : HashMap<String, String> = HashMap::new();

    if let Some(dir) = matches.value_of("dir") {
        let _ = dir::walk(dir.to_string(), &mut files);
        for (key, value) in files {
            println!("{}, {}", key, value);
        }
    } else {
        println!("scan directory not secified.");
    }
}
