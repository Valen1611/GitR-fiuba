use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::fs;
use std::path::Path;

use crate::command_utils::{flate2compress, print_commit_data};
use crate::gitr_errors::GitrError;
use crate::logger;

use flate2::read::ZlibDecoder;



/***************************
 *************************** 
 *      FS FUNCTIONS
 **************************
 **************************/

/// Reads a file and returns the content as String
/// On Error returns a FileReadError
pub fn read_file(path: String) -> Result<String, GitrError> {
    let log_msg = format!("reading data from: {}", path);
    logger::log_file_operation(log_msg)?; 
    match fs::read_to_string(path.clone()) {
        Ok(data) => Ok(data),
        Err(_) => {
            logger::log_error(format!("No se pudo leer: {}", path))?;
            Err(GitrError::FileReadError(path))},
    }
}


// Writes a file with the given text
pub fn write_file(path: String, text: String) -> Result<(), GitrError> {
    let log_msg = format!("writing data to: {}", path);
    logger::log_file_operation(log_msg)?;

    let mut archivo = match File::create(&path) {
        Ok(archivo) => archivo,
        Err(_) => return Err(GitrError::FileCreationError(path)),
    };

    match archivo.write_all(text.as_bytes()) {
        Ok(_) => Ok(()),
        Err(_) => Err(GitrError::FileWriteError(path)),
    }
}


pub fn append_to_file(path: String, text: String) -> Result<(), GitrError> {
    // let log_msg = format!("appending data to: {}", path);
    // logger::log_file_operation(log_msg)?;
    let mut file = match OpenOptions::new()
        .write(true)
        .append(true)
        .open(&path) {
            Ok(file) => file,
            Err(_) => return Err(GitrError::FileWriteError(path)),

        };
    
    match writeln!(file, "{}", text) {
        Ok(_) => Ok(()),
        Err(_) => Err(GitrError::FileWriteError(path)),
    }
}


/// Creates a directory in the current path
/// On Error returns a AlreadyInitialized
fn create_directory(path: &String)->Result<(), GitrError>{
    let log_msg = format!("creating dir: {}", path);
    logger::log_file_operation(log_msg)?; 
    match fs::create_dir(path){
        Ok(_) => Ok(()),
        Err(_) => {
            Err(GitrError::AlreadyInitialized)}
    }
}




/***************************
 *************************** 
 *      GIT OBJECTS
 **************************
 **************************/


/// A diferencia de write_file, esta funcion recibe un vector de bytes
/// como data, y lo escribe en el archivo de path.
pub fn write_compressed_data(path: &str, data: &[u8]) -> Result<(), GitrError>{
    let log_msg = format!("writing data to: {}", path);
    logger::log_file_operation(log_msg)?;
    // let file: File = match File::create(path) {
    //     Ok(file) => file,
    //     Err(_) => return Err(GitrError::FileCreationError(path.to_string())),
    // };

    match fs::write(path, data) {
        Ok(_) => Ok(()),
        Err(_) => Err(GitrError::FileCreationError(path.to_string()))
    }

}

fn read_compressed_file(path: &str) -> Result<Vec<u8>, GitrError> {
    let log_msg = format!("reading data from: {}", path);
    logger::log_file_operation(log_msg)?;
    let file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return Err(GitrError::FileReadError(path.to_string())),
    };
    let mut decoder = ZlibDecoder::new(file);
    let mut buffer = Vec::new();
    match decoder.read_to_end(&mut buffer){
        Ok(_) => Ok(buffer.clone()),
        Err(_) => Err(GitrError::FileReadError(path.to_string())),
    }
}

pub fn init_repository(name: &String) ->  Result<(),GitrError>{
        create_directory(name)?;
        create_directory(&(name.clone() + "/gitr"))?;
        create_directory(&(name.clone() + "/gitr/objects"))?;
        create_directory(&(name.clone() + "/gitr/refs"))?;
        create_directory(&(name.clone() + "/gitr/refs/heads"))?;
    
    Ok(())
}


///read .head_repo and returns the content
pub fn get_current_repo() -> Result<String, GitrError>{
    let current_repo = read_file(".head_repo".to_string())?;
    Ok(current_repo)
}


///receive compressed raw data from a file with his hash and write it in the objects folder
pub fn write_object(data:Vec<u8>, hashed_name:String) -> Result<(), GitrError>{
    let log_msg = format!("writing object {}", hashed_name);
    logger::log_file_operation(log_msg)?;

    let folder_name = hashed_name[0..2].to_string();
    let file_name = hashed_name[2..].to_string();
    let repo = get_current_repo()?;
    let dir = repo + "/gitr/objects/";
    let folder_dir = dir.clone() + &folder_name;

    if fs::metadata(&folder_dir).is_err() {
        create_directory(&folder_dir)?;
    }
    
    write_compressed_data(&(folder_dir.clone() + "/" + &file_name),  &data)?;
    Ok(())
}





pub fn read_object(object: &String) -> Result<String, GitrError>{
    if object.len() < 3{
        return Err(GitrError::ObjectNotFound(object.clone()));
    }
    let folder_name = object[0..2].to_string();
    let file_name = object[2..].to_string();

    let repo = get_current_repo()?;
    let dir = repo + "/gitr/objects/";
    
    let folder_dir = dir.clone() + &folder_name;
    let path = dir + &folder_name +  "/" + &file_name;
    if fs::metadata(folder_dir).is_err(){
        return Err(GitrError::ObjectNotFound(object.clone()));
    }
    if fs::metadata(&path).is_err(){
        return Err(GitrError::ObjectNotFound(object.clone()));
    }
    let data = read_compressed_file(&path);
    let data = match data{
        Ok(data) => data,
        Err(_) => return Err(GitrError::FileReadError(path)),
    };
    let data = String::from_utf8(data);
    let data = match data{
        Ok(data) => data,
        Err(_) => return Err(GitrError::FileReadError(path)),
    };
    Ok(data)
}

pub fn read_index() -> Result<String, GitrError>{
    let repo = get_current_repo()?;
    let path = repo + "/gitr/index";
    let data = read_compressed_file(&path);
    let data = match data{
        Ok(data) => data,
        Err(_) => return Err(GitrError::FileReadError(path)),
    };
    let data = String::from_utf8(data);
    let data = match data{
        Ok(data) => data,
        Err(_) => return Err(GitrError::FileReadError(path)),
    };
    Ok(data)
}

pub fn add_to_index(path: &String, hash: &String) -> Result<(), GitrError>{
    let mut index;
    let repo = get_current_repo()?;
    let new_blob = format!("100644 {} 0 {}", hash, path);
    let dir = repo + "/gitr/index";
    
    if fs::metadata(dir.clone()).is_err(){
        let _ = write_file(dir.clone(), String::from(""));
        index = new_blob;
    }else {
        index = read_index()?;
        let mut overwrited = false;
        for line in index.clone().lines(){
            let attributes = line.split(' ').collect::<Vec<&str>>();

            if attributes[3] == path{
                let log_msg = format!("adding {} to index", path);
                logger::log_action(log_msg)?;
                index = index.replace(line, &new_blob);
                overwrited = true;
                break;
            }

        }
        if !overwrited{
            index = index + "\n" + &new_blob;
        }
    }
    let compressed_index = flate2compress(index)?;
    let _ = write_compressed_data(dir.as_str(), compressed_index.as_slice());
    Ok(())

}


pub fn get_head() ->  Result<String, GitrError>{
    let repo = get_current_repo()?;
    let path = repo + "/gitr/HEAD";
    if fs::metadata(path.clone()).is_err(){
        write_file(path.clone(), String::from("ref: refs/heads/master"))?;
        return Ok("None".to_string())
        // return Err(GitrError::NoHead);
    }
    let head = read_file(path.clone())?;
    let head = head.trim_end().to_string();
    let head = head.split(' ').collect::<Vec<&str>>()[1];
    Ok(head.to_string())
}

pub fn update_head(head: &String) -> Result<(), GitrError>{
    let repo = get_current_repo()?;
    let path = repo + "/gitr/HEAD";
    write_file(path, format!("ref: {}", head))?;
    Ok(())
}


pub fn get_branches()-> Result<Vec<String>, Box<dyn Error>>{
    let mut branches: Vec<String> = Vec::new();
    let repo = get_current_repo()?;
    let dir = repo + "/gitr/refs/heads";
    let paths = fs::read_dir(dir.clone())?;
    for path in paths {
        let path = path?;
        let path = path.path();
        let path = path.to_str();
        let path = match path{
            Some(path) => path,
            None => return Err(Box::new(GitrError::FileReadError(dir))),
        };
        let path = path.split('/').collect::<Vec<&str>>();
        let path = path[path.len()-1];
        branches.push(path.to_string());
    }
    Ok(branches)
}

pub fn get_current_commit()->Result<String, GitrError>{
    let head_path = get_head()?;
    if head_path == "None"{
        return Err(GitrError::NoHead);
    }
    let repo = get_current_repo()?;
    let path = repo + "/gitr/" + &head_path;
    let head = read_file(path)?;
    Ok(head)
}

pub fn delete_branch(branch:String, moving: bool)-> Result<(), GitrError>{
    let repo = get_current_repo()?;
    let path = format!("{}/gitr/refs/heads/{}", repo, branch);
    let head = get_head()?;
    if moving {
        let _ = fs::remove_file(path);
        return Ok(())
    }
    let current_head = repo + "/gitr/" + &head;
    if current_head== path || head == "None"{
        println!("cannot delete branch '{}': HEAD points to it", branch);
        return Ok(())
    }
    let _ = fs::remove_file(path);
    println!("Deleted branch {}", branch);
    Ok(())
}

pub fn move_branch(old_branch: String, new_branch: String) -> Result<(), Box<dyn Error>> {
    fs::rename(old_branch, new_branch)?;
    Ok(())
}   
pub fn get_commit(branch:String)->Result<String, GitrError>{
    let repo = get_current_repo()?;
    let path = format!("{}/gitr/refs/heads/{}",repo, branch);
    let commit = read_file(path.clone())?;
    Ok(commit)
}

pub fn create_tree(path: String, hash: String) -> Result<(), Box<dyn Error>> {
    let repo = get_current_repo()?;
    
    let folder_path = repo.clone() + "/" + &path;
    
    println!("carpeta que deberia crear: {}", folder_path);
    fs::create_dir(folder_path.clone())?;
    let tree_raw_data = read_object(&hash)?;
    
    //let tree_entries = tree_raw_data.split("\0").collect::<Vec<&str>>()[1];
    let raw_data_index = match tree_raw_data.find('\0') {
        Some(index) => index,
        None => {
            println!("Error: invalid object type");
            return Ok(())
        }
    };

    let raw_data = &tree_raw_data[(raw_data_index + 1)..];


    for entry in raw_data.split('\n') {
        let object = entry.split(' ').collect::<Vec<&str>>()[0];
        if object == "100644"{ //blob
            create_blob(entry.to_string())?;

        } else { //tree
            let _new_path_hash = entry.split(' ').collect::<Vec<&str>>()[1];
            let new_path = _new_path_hash.split('\0').collect::<Vec<&str>>()[0]; 
            let hash = _new_path_hash.split('\0').collect::<Vec<&str>>()[1];
            create_tree(folder_path.clone() + "/" + new_path, hash.to_string())?;
        }
    }

    Ok(())
}

pub fn create_blob(entry: String) -> Result<(), Box<dyn Error>> {
    let _blob_path_hash = entry.split(' ').collect::<Vec<&str>>()[1];
    let blob_path = _blob_path_hash.split('\0').collect::<Vec<&str>>()[0];
    let blob_hash = _blob_path_hash.split('\0').collect::<Vec<&str>>()[1];
    //println!("blob hash: {}", blob_hash);
    let new_blob = read_object(&(blob_hash.to_string()))?;
    
    let new_blob_only_data = new_blob.split('\0').collect::<Vec<&str>>()[1];
    write_file(blob_path.to_string(), new_blob_only_data.to_string())?;
    Ok(())
}

/*
///tree <size-of-tree-in-bytes>\0
// <file-1-mode> path\0<hash
// <file-2-mode> path\0<file-2-blob-hash>
// ...
// <file-n-mode> <file-n-path>\0<file-n-blob-hash>
*/ 
pub fn update_working_directory(commit: String)-> Result<(), Box<dyn Error>>{
    let _ = delete_all_files();
    let main_tree = get_main_tree(commit)?;
    let tree = read_object(&main_tree)?;
    //let tree_entries = tree.split("\0").collect::<Vec<&str>>()[1];
    
    // REEMPLAZAR ESTO POR SPLIT_ONCE
    //tree.split_once('\0')
    let raw_data_index = match tree.find('\0') {
        Some(index) => index,
        None => {
            println!("Error: invalid object type");
            return Ok(())
        }
    };
    let raw_data = &tree[(raw_data_index + 1)..];
    println!("raw data: {}", raw_data);

    for entry in raw_data.split('\n'){
        let object: &str = entry.split(' ').collect::<Vec<&str>>()[0];
        if object == "40000"{
            let _new_path_hash = entry.split(' ').collect::<Vec<&str>>()[1];
            let new_path = _new_path_hash.split('\0').collect::<Vec<&str>>()[0]; 
            let hash = _new_path_hash.split('\0').collect::<Vec<&str>>()[1];
            create_tree(new_path.to_string(), hash.to_string())?;
        } else{
            create_blob(entry.to_string().clone())?;
        }
    }
    Ok(())
}

pub fn get_main_tree(commit:String)->Result<String, Box<dyn Error>>{
    let commit = read_object(&commit)?;
    let commit = commit.split('\n').collect::<Vec<&str>>();
    let tree_base = commit[0].split('\0').collect::<Vec<&str>>()[1];
    let tree = read_object(&(tree_base.to_string()))?;
    let raw_data_index = match tree.find('\0') {
        Some(index) => index,
        None => {
            println!("Error: invalid object type");
            return Ok(".".to_string());
        }
    };
    let raw_data = &tree[(raw_data_index + 1)..];
    let tree_hash = raw_data.split('\0').collect::<Vec<&str>>()[1];
    println!("{}", tree_hash);
    Ok(tree_hash.to_string())
}

pub fn delete_all_files()-> Result<(), Box<dyn Error>>{  
    let repo = get_current_repo()?;
    let path = Path::new(&repo);
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            
                if entry.file_name() != "gitr" {
                    println!("Deleting {:?}", entry.path());
                    let _ = fs::remove_dir_all(entry.path());
                }

            
        }
    }
    Ok(())
}


pub fn update_current_repo(dir_name: &String) -> Result<(), GitrError> {
    write_file(".head_repo".to_string(), dir_name.to_string())?;

    Ok(())
}


pub fn print_commit_log(quantity: String)-> Result<(), GitrError>{
    let mut current_commit = get_current_commit()?;
    let limit = match quantity.parse::<i32>(){
        Ok(quantity) => quantity,
        Err(_) => return Err(GitrError::InvalidArgumentError),
    };
    let mut counter = 0;
    loop{
        counter += 1;
        let commit = read_object(&current_commit)?;
        print_commit_data(&commit);
        println!("\n");
        let commit = commit.split('\n').collect::<Vec<&str>>();
        let parent = commit[1].split(' ').collect::<Vec<&str>>()[1];
        if parent == "None" || counter == limit{
            break;
        }
        current_commit = parent.to_string();
    }

    Ok(())
}

pub fn get_repos() -> Vec<String> {
    let mut repos: Vec<String> = Vec::new();
    if let Ok(entries) = fs::read_dir("./") {
        for entry in entries.flatten() {
            if entry.file_name() == "gitr" || 
                entry.file_name() == "src" ||
                entry.file_name() == "tests" {
                continue;
            }
            if entry.file_type().unwrap().is_dir() {
                println!("{}", entry.path().display());
                repos.push(entry.path().display().to_string()[2..].to_string());
            }
        }
    }
    repos
}
