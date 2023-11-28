use std::{path::Path, fs};
use gitr::command_utils::{get_object_properties, print_branches};
use gitr::{command_utils, file_manager};
use gitr::objects::blob::Blob;
use gitr::{commands::commands, gitr_errors::GitrError};
use gitr::file_manager::{get_current_repo, update_current_repo, write_file, read_index};
use serial_test::serial;


/*********************
      INIT TESTS
*********************/

#[test]
#[serial]
fn test_init() {
    let cliente = "cliente_init".to_string();
    fs::create_dir_all(Path::new(&cliente)).unwrap();
    commands::init(vec!["test_init".to_string()],cliente.clone()).unwrap();
    let directory_path = String::from("cliente_init/test_init/gitr/");
    assert!(Path::new(&(directory_path.clone() + "objects")).is_dir());
    assert!(Path::new(&(directory_path.clone() + "refs")).is_dir());
    assert!(Path::new(&(directory_path.clone() + "refs/heads")).is_dir());
    let current_repo = get_current_repo(cliente.clone()).unwrap();
    assert_eq!("cliente_init/test_init", current_repo);
    fs::remove_dir_all(cliente.clone()).unwrap();
}


#[test]
#[serial]
fn test_init_exists() {
    let cliente = "cliente_init_2".to_string();
    fs::create_dir_all(Path::new(&cliente)).unwrap();
    commands::init(vec!["test_init_exists".to_string()], cliente.clone()).unwrap();
    let res = commands::init(vec!["test_init_exists".to_string()], cliente.clone());
    let error = res.unwrap_err();
    assert!(matches!(error, GitrError::AlreadyInitialized));
    fs::remove_dir_all("cliente_init_2").unwrap();
}

// /*********************
//   ADD TESTS
// *********************/
#[test]
#[serial]
fn test_add(){
    let cliente = "cliente_add".to_string();
    fs::create_dir_all(Path::new(&cliente)).unwrap();
    commands::init(vec!["test_add_blob".to_string()], cliente.clone()).unwrap();
    let _ = write_file((cliente.clone() + "/test_add_blob/blob1").to_string(), "Hello, im blob 1".to_string());
    let _ = write_file((cliente.clone() + "/test_add_blob/blob2").to_string(), "Hello, im blob 2".to_string());
    commands::add(vec!["blob1".to_string()], cliente.clone()).unwrap();
    commands::add(vec!["blob2".to_string()], cliente.clone()).unwrap();
    let hash1 = Blob::new("Hello, im blob 1".to_string()).unwrap().get_hash();
    let hash2 = Blob::new("Hello, im blob 2".to_string()).unwrap().get_hash();
    assert!(Path::new(&(cliente.clone() + "/test_add_blob/gitr/index")).is_file());
    let index = file_manager::read_index(cliente.clone()).unwrap();
    assert!(index.contains(&hash1));
    assert!(index.contains(&hash2));
    let hash1_dir = (cliente.clone() +"/test_add_blob/gitr/objects/").to_string() + &hash1[..2];
    let hash2_dir = (cliente.clone() + "/test_add_blob/gitr/objects/").to_string() + &hash2[..2];
    assert!(Path::new(&hash1_dir).is_dir());
    assert!(Path::new(&hash2_dir).is_dir());
    let hash1_file = hash1_dir.clone() + "/" + &hash1[2..];
    let hash2_file = hash2_dir.clone() + "/" + &hash2[2..];
    assert!(Path::new(&hash1_file).is_file());
    assert!(Path::new(&hash2_file).is_file());
    fs::remove_dir_all(cliente).unwrap();

}

// /*********************
//   RM TESTS
// *********************/
#[test]
#[serial]
fn test_rm(){
    let cliente = "cliente_rm".to_string();
    fs::create_dir_all(Path::new(&cliente)).unwrap();
    commands::init(vec!["test_rm_blob".to_string()], cliente.clone()).unwrap();
    let _ = write_file((cliente.clone() + "/test_rm_blob/blob1").to_string(), "Hello, im blob 1".to_string());
    let _ = write_file((cliente.clone() + "/test_rm_blob/blob2").to_string(), "Hello, im blob 2".to_string());
    commands::add(vec!["blob1".to_string()], cliente.clone()).unwrap();
    commands::add(vec!["blob2".to_string()], cliente.clone()).unwrap();
    let hash1 = Blob::new("Hello, im blob 1".to_string()).unwrap().get_hash();
    let hash2 = Blob::new("Hello, im blob 2".to_string()).unwrap().get_hash();
    assert!(Path::new(&(cliente.clone() + "/test_rm_blob/gitr/index")).is_file());
    let index = file_manager::read_index(cliente.clone()).unwrap();
    assert!(index.contains(&hash1));
    assert!(index.contains(&hash2));
    let hash1_dir = (cliente.clone() + "/test_rm_blob/gitr/objects/").to_string() + &hash1[..2];
    let hash2_dir = (cliente.clone() + "/test_rm_blob/gitr/objects/").to_string() + &hash2[..2];
    assert!(Path::new(&hash1_dir).is_dir());
    assert!(Path::new(&hash2_dir).is_dir());
    let hash1_file = hash1_dir.clone() + "/" + &hash1[2..];
    let hash2_file = hash2_dir.clone() + "/" + &hash2[2..];
    assert!(Path::new(&hash1_file).is_file());
    assert!(Path::new(&hash2_file).is_file());
    commands::rm(vec!["blob1".to_string()], cliente.clone()).unwrap();
    commands::rm(vec!["blob2".to_string()], cliente.clone()).unwrap();
    let index = file_manager::read_index(cliente.clone()).unwrap();
    assert!(!index.contains(&hash1));
    assert!(!index.contains(&hash2));
    fs::remove_dir_all(cliente).unwrap();

}

// /*********************
//   LS_FILES TESTS
// *********************/
#[test]
#[serial]
fn test_ls_files_returns_empty_after_init(){
    let cliente = "cliente_ls_files_1".to_string();
    fs::create_dir_all(Path::new(&cliente)).unwrap();
    commands::init(vec!["test_ls_files_empty".to_string()], cliente.clone()).unwrap();
    let res = command_utils::get_ls_files_cached(cliente.clone()).unwrap();
    assert!(res.is_empty());
    fs::remove_dir_all(cliente.clone()).unwrap();
}

#[test]
#[serial]
fn test_ls_files_stage_after_adding_files(){
    let cliente = "cliente_ls_files_2".to_string();
    fs::create_dir_all(Path::new(&cliente)).unwrap();
    commands::init(vec!["test_ls_files_stage".to_string()], cliente.clone()).unwrap();
    let _ = write_file((cliente.clone() + "/test_ls_files_stage/blob1").to_string(), "Hello, im blob 1".to_string());
    let _ = write_file((cliente.clone() + "/test_ls_files_stage/blob2").to_string(), "Hello, im blob 2".to_string());
    commands::add(vec!["blob1".to_string()],cliente.clone()).unwrap();
    commands::add(vec!["blob2".to_string()], cliente.clone()).unwrap();
    let res = read_index(cliente.clone()).unwrap();
    let correct_res = String::from("100644 016a41a6a35d50d311286359f1a7611948a9c529 0 cliente_ls_files_2/test_ls_files_stage/blob1\n100644 18d74b139e1549bb6a96b281e6ac3a0ec9e563e8 0 cliente_ls_files_2/test_ls_files_stage/blob2");
    fs::remove_dir_all(cliente.clone()).unwrap();
    assert_eq!(res, correct_res);
}

/*********************
  TAG TESTS
*********************/
#[test]
#[serial]
fn test_tag_lightweight(){
    let cliente = "cliente_tag_lightweight".to_string();
    fs::create_dir_all(Path::new(&cliente)).unwrap();
    commands::init(vec!["test_tag_lightweight".to_string()], cliente.clone()).unwrap();
    let _ = write_file((cliente.clone() + "/gitrconfig").to_string(), "[user]\n\tname = test\n\temail = test@gmail.com".to_string());
    let _ = write_file((cliente.clone() + "/test_tag_lightweight/blob1").to_string(), "Hello, im blob 1".to_string());
    let _ = write_file((cliente.clone() + "/test_tag_lightweight/blob2").to_string(), "Hello, im blob 2".to_string());
    commands::add(vec!["blob1".to_string()], cliente.clone()).unwrap();
    commands::add(vec!["blob2".to_string()], cliente.clone()).unwrap();
    commands::commit(vec!["-m".to_string(), "\"commit 1\"".to_string()], cliente.clone()).unwrap();
    commands::tag(vec!["tag1".to_string()], cliente.clone()).unwrap();
    let res = file_manager::read_file(cliente.clone() + "/test_tag_lightweight/gitr/refs/tags/tag1").unwrap();
    let current_commit = file_manager::get_current_commit(cliente.clone()).unwrap();
    assert_eq!(res, current_commit);
    fs::remove_dir_all(cliente.clone()).unwrap();
}

#[test]
#[serial]
fn test_tag_annotated(){
    let cliente = "cliente_tag_annotated".to_string();
    fs::create_dir_all(Path::new(&cliente)).unwrap();
    commands::init(vec!["test_tag_annotated".to_string()], cliente.clone()).unwrap();
    let _ = write_file((cliente.clone() + "/gitrconfig").to_string(), "[user]\n\tname = test\n\temail = test@gmail.com".to_string());
    let _ = write_file((cliente.clone() + "/test_tag_annotated/blob1").to_string(), "Hello, im blob 1".to_string());
    let _ = write_file((cliente.clone() + "/test_tag_annotated/blob2").to_string(), "Hello, im blob 2".to_string());
    commands::add(vec!["blob1".to_string()], cliente.clone()).unwrap();
    commands::add(vec!["blob2".to_string()], cliente.clone()).unwrap();
    commands::commit(vec!["-m".to_string(), "\"commit 1\"".to_string()], cliente.clone()).unwrap();
    commands::tag(vec!["-a".to_string(), "tag1".to_string(), "-m".to_string(), "\"un tag anotado\"".to_string()], cliente.clone()).unwrap();
    let res = file_manager::read_file(cliente.clone() + "/test_tag_annotated/gitr/refs/tags/tag1").unwrap();
    let object =  file_manager::read_object(&res,file_manager::get_current_repo(cliente.clone()).unwrap(), true).unwrap();
    let object_type = object.split(' ').collect::<Vec<&str>>()[0];
    assert_eq!(object_type, "tag");
    fs::remove_dir_all(cliente.clone()).unwrap();
}

#[test]
#[serial]
fn test_tag_delete(){
    let cliente = "cliente_tag_delete".to_string();
    fs::create_dir_all(Path::new(&cliente)).unwrap();
    commands::init(vec!["test_tag_delete".to_string()], cliente.clone()).unwrap();
    let _ = write_file((cliente.clone() + "/gitrconfig").to_string(), "[user]\n\tname = test\n\temail = test@gmail.com".to_string());
    let _ = write_file((cliente.clone() + "/test_tag_delete/blob1").to_string(), "Hello, im blob 1".to_string());
    let _ = write_file((cliente.clone() + "/test_tag_delete/blob2").to_string(), "Hello, im blob 2".to_string());
    commands::add(vec!["blob1".to_string()], cliente.clone()).unwrap();
    commands::add(vec!["blob2".to_string()], cliente.clone()).unwrap();
    commands::commit(vec!["-m".to_string(), "\"commit 1\"".to_string()], cliente.clone()).unwrap();
    commands::tag(vec!["tag1".to_string()], cliente.clone()).unwrap();
    let res = file_manager::read_file(cliente.clone() + "/test_tag_delete/gitr/refs/tags/tag1").unwrap();
    let current_commit = file_manager::get_current_commit(cliente.clone()).unwrap();
    assert_eq!(res, current_commit);
    commands::tag(vec!["-d".to_string(), "tag1".to_string()], cliente.clone()).unwrap();
    let res = file_manager::read_file(cliente.clone() + "/test_tag_delete/gitr/refs/tags/tag1");
    assert!(res.is_err());
    fs::remove_dir_all(cliente.clone()).unwrap();
}

// /*********************
//   BRANCH TESTS
// *********************/
#[test]
#[serial]
fn test_branch_newbranch(){
    let cliente = "cliente_branch".to_string();
    fs::create_dir_all(Path::new(&cliente)).unwrap();
    commands::init(vec!["test_branch".to_string()], cliente.clone()).unwrap();
    let _ = write_file((cliente.clone() + "/gitrconfig").to_string(),("[user]\n\tname = test\n\temail = test@gmail.com").to_string());
    let _ = write_file((cliente.clone() + "/test_branch/blob1").to_string(), "Hello, im blob 1".to_string());
    let _ = write_file((cliente.clone() + "/test_branch/blob2").to_string(), "Hello, im blob 2".to_string());
    commands::add(vec!["blob1".to_string()], cliente.clone()).unwrap();
    commands::add(vec!["blob2".to_string()], cliente.clone()).unwrap();
    commands::commit(vec!["-m".to_string(), "\"commit 1\"".to_string()], cliente.clone()).unwrap();
    commands::branch(vec!["branch1".to_string()], cliente.clone()).unwrap();
    let res = print_branches(cliente.clone()).unwrap();
    let correct_res = String::from("* \x1b[92mmaster\x1b[0m\nbranch1");
    assert_eq!(res, correct_res);
    fs::remove_dir_all(cliente.clone()).unwrap();
}

#[test]
#[serial]
fn test_branch_no_commit(){
    let cliente = "cliente_branch_no_commit".to_string();
    fs::create_dir_all(Path::new(&cliente)).unwrap();
    commands::init(vec!["test_branch_no_commit".to_string()], cliente.clone()).unwrap();
    let error = commands::branch(vec!["branch1".to_string()], cliente.clone()).unwrap_err();
    let res = print_branches(cliente.clone()).unwrap();
    let correct_res = String::from("");
    assert_eq!(res, correct_res);
    assert!(matches!(error, GitrError::NoCommitExisting(_)));
    fs::remove_dir_all(cliente.clone()).unwrap();
}

#[test]
#[serial]
fn test_branch_already_exists(){
    let cliente = "cliente_branch_already_exists".to_string();
    fs::create_dir_all(Path::new(&cliente)).unwrap();
    commands::init(vec!["test_branch_already_exists".to_string()], cliente.clone()).unwrap();
    let _ = write_file((cliente.clone() + "/gitrconfig").to_string(),("[user]\n\tname = test\n\temail =test@gmail.com").to_string());
    let _ = write_file((cliente.clone() + "/test_branch_already_exists/blob1").to_string(), "Hello, im blob 1".to_string());
    let _ = write_file((cliente.clone() + "/test_branch_already_exists/blob2").to_string(), "Hello, im blob 2".to_string());
    commands::add(vec!["blob1".to_string()], cliente.clone()).unwrap();
    commands::add(vec!["blob2".to_string()], cliente.clone()).unwrap();
    commands::commit(vec!["-m".to_string(), "\"commit 1\"".to_string()], cliente.clone()).unwrap();
    commands::branch(vec!["branch1".to_string()], cliente.clone()).unwrap();
    let error = commands::branch(vec!["branch1".to_string()], cliente.clone()).unwrap_err();
    assert!(matches!(error, GitrError::BranchAlreadyExistsError(_)));
    fs::remove_dir_all(cliente.clone()).unwrap();
}

#[test]
#[serial]
fn test_branch_delete(){
    let cliente = "cliente_branch_delete".to_string();
    fs::create_dir_all(Path::new(&cliente)).unwrap();
    commands::init(vec!["test_branch_delete".to_string()], cliente.clone()).unwrap();
    let _ = write_file((cliente.clone() + "/gitrconfig").to_string(),("[user]\n\tname = test\n\temail =test@gmail.com").to_string());
    let _ = write_file((cliente.clone() + "/test_branch_delete/blob1").to_string(), "Hello, im blob 1".to_string());
    let _ = write_file((cliente.clone() + "/test_branch_delete/blob2").to_string(), "Hello, im blob 2".to_string());
    commands::add(vec!["blob1".to_string()], cliente.clone()).unwrap();
    commands::add(vec!["blob2".to_string()], cliente.clone()).unwrap();
    commands::commit(vec!["-m".to_string(), "\"commit 1\"".to_string()], cliente.clone()).unwrap();
    commands::branch(vec!["branch1".to_string()], cliente.clone()).unwrap();
    let res = print_branches(cliente.clone()).unwrap();
    let correct_res = String::from("* \x1b[92mmaster\x1b[0m\nbranch1");
    assert_eq!(res, correct_res);
    commands::branch(vec!["-d".to_string(), "branch1".to_string()], cliente.clone()).unwrap();
    let res = print_branches(cliente.clone()).unwrap();
    let correct_res = String::from("* \x1b[92mmaster\x1b[0m\n");
    assert_eq!(res, correct_res);
    fs::remove_dir_all(cliente.clone()).unwrap();
}

#[test]
#[serial]
fn test_branch_delete_current(){
    let cliente = "cliente_branch_delete_current".to_string();
    fs::create_dir_all(Path::new(&cliente)).unwrap();
    commands::init(vec!["test_branch_delete_current".to_string()], cliente.clone()).unwrap();
    let _ = write_file((cliente.clone() + "/gitrconfig").to_string(),("[user]\n\tname = test\n\temail =test@gmail.com").to_string());
    let _ = write_file((cliente.clone() + "/test_branch_delete_current/blob1").to_string(), "Hello, im blob 1".to_string());
    let _ = write_file((cliente.clone() + "/test_branch_delete_current/blob2").to_string(), "Hello, im blob 2".to_string());
    commands::add(vec!["blob1".to_string()], cliente.clone()).unwrap();
    commands::add(vec!["blob2".to_string()], cliente.clone()).unwrap();
    commands::commit(vec!["-m".to_string(), "\"commit 1\"".to_string()], cliente.clone()).unwrap();
    let error = commands::branch(vec!["-d".to_string(), "master".to_string()], cliente.clone()).unwrap_err();

}