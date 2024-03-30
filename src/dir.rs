use std::fs;
use std::io;
use std::collections::HashMap;

pub fn walk(dir: String, files : &mut HashMap<String, String>) -> Result<(), io::Error> {
    let entries = fs::read_dir(dir)?;
    for entry in entries {
        let entry    = entry?;
        let metadata = entry.metadata()?;
        let path     = entry.path().display().to_string();
        if metadata.is_dir() {
            let _ = walk(path, files);
        } else {
            files.insert(path, String::from("test"));
        }
    }
    Ok(())
}
