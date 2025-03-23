use std::path::Path;

#[derive(Debug)]
pub struct FileInfo {
    pub container: Option<String>,
    pub path: String,
    pub crc: u32,
}

impl FileInfo {
    pub fn new(path: &str, _length: u64, crc: u32) -> Self {
        let parts: Vec<&str> = path.split('\t').collect();
        let (container, path) = if parts.len() == 2 {
            (Some(parts[0].to_string()), parts[1].to_string())
        } else {
            (None, parts[0].to_string())
        };
        Self { container, path, crc }
    }

    pub fn get_extension(&self) -> Option<String> {
        Path::new(&self.path)
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
    }
}

#[derive(Debug)]
pub struct DuplicateGroup {
    pub files: Vec<FileInfo>,
}

impl DuplicateGroup {
    pub fn new(_length: u64, _crc: u32, files: Vec<FileInfo>) -> Self {
        Self { files }
    }

    pub fn sort_files(&mut self) {
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

pub fn calc_ext_prior(path: &str) -> u8 {
    let path_lower = path.to_lowercase();
    match path_lower.as_str() {
        p if p.ends_with(".zip") => 8,
        p if p.ends_with(".cab") => 6,
        _ => 0,
    }
} 