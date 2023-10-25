use std::{path::Path, fs};
use gitr_cliente::commands::commands;

#[test]
fn test_init() {
    commands::init(vec!["test_init".to_string()]).unwrap();
    let directory_path = String::from("test_init/gitr/");
    assert!(Path::new(&(directory_path.clone() + "objects")).is_dir());
    assert!(Path::new(&(directory_path.clone() + "refs")).is_dir());
    assert!(Path::new(&(directory_path.clone() + "refs/heads")).is_dir());
    
    fs::remove_dir_all("test_init").unwrap();
}