use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use tempfile::tempdir;

use rmdup::archiver;

fn test_file(path: &str) -> String
{
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets").join("test_data").join(path);
    path.display().to_string()
}

#[test]
fn crc() {
    let zip_file = test_file("test.zip");
    let actual   = archiver::make_crc_from_path(&zip_file).unwrap();
    let expected = 4147033919;
    assert_eq!(expected, actual);

    let zip_file = test_file("test.lzh");
    let actual   = archiver::make_crc_from_path(&zip_file).unwrap();
    let expected = 2114793262;
    assert_eq!(expected, actual);

    let zip_file = test_file("test.rar");
    let actual   = archiver::make_crc_from_path(&zip_file).unwrap();
    let expected = 3022389044;
    assert_eq!(expected, actual);
}

#[test]
fn cabinet() {
    let arc_file = test_file("test.cab");
    let temp_dir = tempdir().unwrap();
	let dir = temp_dir.path().to_str().unwrap().to_string();
    env::set_var("RMDUP_HOME", &dir);
    
    let mut file = PathBuf::from(dir);
    file.push(Path::new(&arc_file).file_name().unwrap());
    std::fs::copy(arc_file, &file).unwrap();
    let file = file.to_str().unwrap();

    let mut map_len = HashMap::new();
    let mut map_crc = HashMap::new();

    archiver::cabinet::walk(&file, &mut map_len, &mut map_crc).unwrap();

    assert_eq!(2, map_len.len());
    assert_eq!(4, map_len.get(&3).unwrap().len());
    assert_eq!(4, map_len.get(&6).unwrap().len());
    assert_eq!(0, map_crc.len());

    let mut container = "";
    let mut remove_file = vec![];
    for path in map_len.get(&6).unwrap() {
        let p: Vec<_> = path.split("\t").collect();
        let crc = archiver::cabinet::crc(p[0], p[1]).unwrap();
        if p[1].contains("first.txt"){
            assert_eq!(3131343897, crc);
            container = p[0];
            remove_file.push(p[1].to_string());
        } else if p[1].contains("second.txt"){
            assert_eq!(645441646, crc);
        }
    }

    archiver::cabinet::remove(container, remove_file).unwrap();

    let mut map_len = HashMap::new();
    let mut map_crc = HashMap::new();
    archiver::cabinet::walk(&file, &mut map_len, &mut map_crc).unwrap();

    assert_eq!(2, map_len.len());
    assert_eq!(4, map_len.get(&3).unwrap().len());
    assert_eq!(3, map_len.get(&6).unwrap().len());
    assert_eq!(0, map_crc.len());
}

#[test]
fn lzh() {
    let cab_file = test_file("test.lzh");
    
    let mut map_len = HashMap::new();
    let mut map_crc = HashMap::new();

    archiver::lzh::walk(cab_file.as_str(), &mut map_len, &mut map_crc).unwrap();

    assert_eq!(3, map_len.len());
    assert_eq!(4, map_len.get(&3).unwrap().len());
    assert_eq!(4, map_len.get(&6).unwrap().len());
    assert_eq!(0, map_crc.len());

    for path in map_len.get(&6).unwrap() {
        let p: Vec<_> = path.split("\t").collect();
        let crc = archiver::lzh::crc(p[0], p[1]).unwrap();
        if p[1].contains("first.txt"){
            assert_eq!(3131343897, crc);
        } else if p[1].contains("second.txt"){
            assert_eq!(645441646, crc);
        }
    }
}
