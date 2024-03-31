use std::collections::HashMap;

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
