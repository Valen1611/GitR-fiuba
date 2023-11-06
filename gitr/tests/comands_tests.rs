use std::{path::Path, fs};
use gitr::{command_utils, file_manager};
use gitr::objects::blob::Blob;
use gitr::{commands::commands, gitr_errors::GitrError};
use gitr::file_manager::{get_current_repo, update_current_repo, write_file};
use serial_test::serial;


/*********************
      INIT TESTS
*********************/

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

/*********************
  ADD TESTS
*********************/
#[test]
#[serial]
fn test_add(){
    let last_repo = get_current_repo().unwrap();
    commands::init(vec!["test_add_blob".to_string()]).unwrap();
    let _ = write_file("test_add_blob/blob1".to_string(), "Hello, im blob 1".to_string());
    let _ = write_file("test_add_blob/blob2".to_string(), "Hello, im blob 2".to_string());
    commands::add(vec!["blob1".to_string()]).unwrap();
    commands::add(vec!["blob2".to_string()]).unwrap();
    let hash1 = Blob::new("Hello, im blob 1".to_string()).unwrap().get_hash();
    let hash2 = Blob::new("Hello, im blob 2".to_string()).unwrap().get_hash();
    assert!(Path::new("test_add_blob/gitr/index").is_file());
    let index = file_manager::read_index().unwrap();
    assert!(index.contains(&hash1));
    assert!(index.contains(&hash2));
    let hash1_dir = "test_add_blob/gitr/objects/".to_string() + &hash1[..2];
    let hash2_dir = "test_add_blob/gitr/objects/".to_string() + &hash2[..2];
    assert!(Path::new(&hash1_dir).is_dir());
    assert!(Path::new(&hash2_dir).is_dir());
    let hash1_file = hash1_dir.clone() + "/" + &hash1[2..];
    let hash2_file = hash2_dir.clone() + "/" + &hash2[2..];
    assert!(Path::new(&hash1_file).is_file());
    assert!(Path::new(&hash2_file).is_file());
    update_current_repo(&last_repo).unwrap();
    fs::remove_dir_all("test_add_blob").unwrap();

}

/*********************
  RM TESTS
*********************/
#[test]
#[serial]
fn test_rm(){
    let last_repo = get_current_repo().unwrap();
    commands::init(vec!["test_rm_blob".to_string()]).unwrap();
    let _ = write_file("test_rm_blob/blob1".to_string(), "Hello, im blob 1".to_string());
    let _ = write_file("test_rm_blob/blob2".to_string(), "Hello, im blob 2".to_string());
    commands::add(vec!["blob1".to_string()]).unwrap();
    commands::add(vec!["blob2".to_string()]).unwrap();
    let hash1 = Blob::new("Hello, im blob 1".to_string()).unwrap().get_hash();
    let hash2 = Blob::new("Hello, im blob 2".to_string()).unwrap().get_hash();
    assert!(Path::new("test_rm_blob/gitr/index").is_file());
    let index = file_manager::read_index().unwrap();
    assert!(index.contains(&hash1));
    assert!(index.contains(&hash2));
    let hash1_dir = "test_rm_blob/gitr/objects/".to_string() + &hash1[..2];
    let hash2_dir = "test_rm_blob/gitr/objects/".to_string() + &hash2[..2];
    assert!(Path::new(&hash1_dir).is_dir());
    assert!(Path::new(&hash2_dir).is_dir());
    let hash1_file = hash1_dir.clone() + "/" + &hash1[2..];
    let hash2_file = hash2_dir.clone() + "/" + &hash2[2..];
    assert!(Path::new(&hash1_file).is_file());
    assert!(Path::new(&hash2_file).is_file());
    commands::rm(vec!["blob1".to_string()]).unwrap();
    commands::rm(vec!["blob2".to_string()]).unwrap();
    let index = file_manager::read_index().unwrap();
    assert!(!index.contains(&hash1));
    assert!(!index.contains(&hash2));
    update_current_repo(&last_repo).unwrap();
    fs::remove_dir_all("test_rm_blob").unwrap();

}