use std::{path::Path, fs};
use gitr_cliente::{commands::commands, gitr_errors::GitrError};
use gitr_cliente::file_manager::{get_current_repo, update_current_repo};
use serial_test::serial;

#[test]
#[serial]
fn test_init() {
    commands::init(vec!["test_init".to_string()]).unwrap();
    let directory_path = String::from("test_init/gitr/");
    let last_repo = get_current_repo().unwrap();
    assert!(Path::new(&(directory_path.clone() + "objects")).is_dir());
    assert!(Path::new(&(directory_path.clone() + "refs")).is_dir());
    assert!(Path::new(&(directory_path.clone() + "refs/heads")).is_dir());
    let current_repo = get_current_repo().unwrap();
    assert_eq!("test_init", current_repo);
    update_current_repo(&last_repo).unwrap();
    fs::remove_dir_all("test_init").unwrap();
}


#[test]
#[serial]
fn test_init_exists() {
    let last_repo = get_current_repo().unwrap();
    commands::init(vec!["test_init_exists".to_string()]).unwrap();
    let res = commands::init(vec!["test_init_exists".to_string()]);
    let error = res.unwrap_err();
    assert!(matches!(error, GitrError::AlreadyInitialized));
    update_current_repo(&last_repo).unwrap();
    fs::remove_dir_all("test_init_exists").unwrap();
}