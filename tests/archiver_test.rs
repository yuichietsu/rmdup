use std::path::PathBuf;

use rmdup::archiver;

fn test_file(path: &str) -> String
{
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets").join("test_data").join(path);
    path.display().to_string()
}

#[test]
fn make_crc() {
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
