use std::{path::Path, fs};
use gitr_cliente::command_utils;
use gitr_cliente::{commands::commands, gitr_errors::GitrError};
use gitr_cliente::file_manager::{get_current_repo, update_current_repo};
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
  HASH-OBJECT TESTS
*********************/

#[test]
#[serial]
fn test_hash_object() {
    // init a repo for the command
    let repo_path = String::from("testing_hash_object_repo");
    commands::init(vec![repo_path.clone()]).unwrap();

    // test the command
    test_hash_object_file(repo_path.clone());


    // remove the repo
    fs::remove_dir_all("testing_hash_object_repo").unwrap();
}
fn test_hash_object_file(repo_path: String) {
    // creamos un archivo para pasarle al comando
    let file_path = String::from("test_hash_object_print.txt");
    let data = String::from("hello world");
    fs::write(&file_path, &data).unwrap();

    // lo hasheamos
    let hash = command_utils::sha1hashing(data);
    let expected_hash = hash.iter().map(|b| format!("{:02x}", b)).collect::<String>();

    // corremos el comando
    let res = commands::hash_object(vec!["-w".to_string(), file_path.clone()]).unwrap();

    // verificamos que el comando haya funcionado

    let expected_folder = repo_path.clone() + "/gitr/objects/" + &expected_hash[..2];
    let expected_file = expected_folder.clone() + "/" + &expected_hash[2..];

    assert!(Path::new(&expected_folder).is_dir());
    assert!(Path::new(&expected_file).is_file());
    
    let hashed_file_data = fs::read_to_string(expected_file).unwrap();

    assert_eq!(hashed_file_data, expected_hash);

} 