use rmdup::file_info::{FileInfo, DuplicateGroup, calc_ext_prior};

#[test]
fn test_file_info_creation_normal_path() {
    let file_info = FileInfo::new("test.txt", 0, 12345);
    assert_eq!(file_info.path, "test.txt");
    assert_eq!(file_info.container, None);
    assert_eq!(file_info.crc, 12345);
}

#[test]
fn test_file_info_creation_with_container() {
    let file_info = FileInfo::new("archive.zip\ttest.txt", 0, 12345);
    assert_eq!(file_info.path, "test.txt");
    assert_eq!(file_info.container, Some("archive.zip".to_string()));
    assert_eq!(file_info.crc, 12345);
}

#[test]
fn test_file_info_get_extension() {
    let file_info = FileInfo::new("test.TXT", 0, 12345);
    assert_eq!(file_info.get_extension(), Some("txt".to_string()));

    let file_info = FileInfo::new("test", 0, 12345);
    assert_eq!(file_info.get_extension(), None);
}

#[test]
fn test_duplicate_group_sorting() {
    let files = vec![
        FileInfo::new("test.txt", 0, 1),
        FileInfo::new("archive.zip\tfile.txt", 0, 2),
        FileInfo::new("data.cab\tfile.txt", 0, 3),
        FileInfo::new("other.txt", 0, 4),
    ];

    let mut group = DuplicateGroup::new(0, 0, files);
    group.sort_files();

    // ZIPが最優先、次にCAB、最後に通常ファイル
    assert_eq!(group.files[0].container, Some("archive.zip".to_string()));
    assert_eq!(group.files[1].container, Some("data.cab".to_string()));
    assert!(group.files[2].container.is_none());
    assert!(group.files[3].container.is_none());
}

#[test]
fn test_calc_ext_prior() {
    assert_eq!(calc_ext_prior("test.zip"), 8);
    assert_eq!(calc_ext_prior("test.ZIP"), 8);
    assert_eq!(calc_ext_prior("test.Zip"), 8);
    assert_eq!(calc_ext_prior("test.cab"), 6);
    assert_eq!(calc_ext_prior("test.CAB"), 6);
    assert_eq!(calc_ext_prior("test.Cab"), 6);
    assert_eq!(calc_ext_prior("test.txt"), 0);
    assert_eq!(calc_ext_prior("test"), 0);
} 