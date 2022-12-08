use stele::utils::library::find_library_path;

use crate::common;

#[test]
fn test_find_library_path_when_at_library_expect_path() {
    let library_path = common::get_test_library_path();
    let actual = find_library_path(&library_path).unwrap();
    let expected = library_path;
    assert_eq!(actual, expected);
}

#[test]
fn test_find_library_path_when_in_library_expect_library_path() {
    let library_path = common::get_test_library_path();
    let cwd = library_path.join("test");
    let actual = find_library_path(&cwd).unwrap();
    let expected = library_path;
    assert_eq!(actual, expected);
}

#[test]
fn test_find_library_path_when_nonexistant_path_expect_error() {
    let library_path = common::get_test_library_path();
    let cwd = library_path.join("does_not_exist");
    let actual_err = find_library_path(&cwd).unwrap_err();
    let actual = format!("{}", actual_err);
    let expected = "(os error 2)";
    assert_eq!(&actual[actual.len() - 12..], expected);
}

#[test]
fn test_find_library_path_when_not_in_library_expect_error() {
    let library_path = common::get_test_library_path();
    let cwd = library_path.parent().unwrap();
    let actual_err = find_library_path(cwd).unwrap_err();
    let actual = format!("{}", actual_err);
    let expected =
        "is not inside a Stele Library. Run `stele init` to create a library at this location.";
    assert_eq!(&actual[actual.len() - 85..], expected);
}
