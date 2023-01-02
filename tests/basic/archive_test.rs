use stele::utils::archive::find_archive_path;

use crate::common;

#[test]
fn test_find_archive_path_when_at_archive_expect_path() {
    let archive_path = common::get_test_archive_path();
    let actual = find_archive_path(&archive_path).unwrap();
    let expected = archive_path;
    assert_eq!(actual, expected);
}

#[test]
fn test_find_archive_path_when_in_archive_expect_archive_path() {
    let archive_path = common::get_test_archive_path();
    let cwd = archive_path.join("test");
    let actual = find_archive_path(&cwd).unwrap();
    let expected = archive_path;
    assert_eq!(actual, expected);
}

#[test]
fn test_find_archive_path_when_nonexistant_path_expect_error() {
    let archive_path = common::get_test_archive_path();
    let cwd = archive_path.join("does_not_exist");
    let actual = find_archive_path(&cwd).unwrap_err();
    let expected = "(os error 2)";
    assert!(
        actual.to_string().contains(expected),
        "\"{actual}\" doesn't contain {expected}"
    );
}

#[test]
fn test_find_archive_path_when_not_in_archive_expect_error() {
    let archive_path = common::get_test_archive_path();
    let cwd = archive_path.parent().unwrap();
    let actual = find_archive_path(cwd).unwrap_err();
    let expected =
        "is not inside a Stele Archive. Run `stele init` to create a archive at this location.";
    assert!(
        actual.to_string().contains(expected),
        "\"{actual}\" doesn't contain {expected}"
    );
}
