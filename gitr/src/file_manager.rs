use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{prelude::*, Bytes};
use std::fs;
use std::path::Path;

use crate::command_utils::flate2compress;
use crate::gitr_errors::GitrError;
use crate::{logger, file_manager};

use chrono::{Utc, TimeZone, FixedOffset};
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
pub fn create_directory(path: &String)->Result<(), GitrError>{
    let log_msg = format!("creating dir: {}", path);
    logger::log_file_operation(log_msg)?; 

    println!("creating dir: {}", path);

    match fs::create_dir(path){
        Ok(_) => Ok(()),
        Err(_) => {
            Err(GitrError::AlreadyInitialized)}
    }
}

pub fn delete_all_files()-> Result<(), GitrError>{  
    let repo = get_current_repo()?;
    match fs::remove_file(repo.clone() + "/gitr/index"){
        Ok(_) => (),
        Err(_) => return Err(GitrError::FileWriteError(repo + "/gitr/index")),
    };
    let path = Path::new(&repo);
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
                if entry.file_name() != "gitr" && entry.file_name() != ".git" {
                    println!("Deleting {:?}", entry.path());
                    
                    if entry.path().is_file() {
                        match fs::remove_file(entry.path()) {
                            Ok(_) => continue,
                            Err(_) => return Err(GitrError::FileWriteError(entry.path().display().to_string())),
                        };
                    }


                    match fs::remove_dir_all(entry.path()) {
                        Ok(_) => (),
                        Err(_) => return Err(GitrError::FileWriteError(entry.path().display().to_string())),
                    };

                    


                }
        }
    }
    Ok(())
}




/***************************
 *************************** 
 *      GIT OBJECTS
 **************************
 *************************
 * reading
 * writing
 * others
 */

// ***reading***
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


pub fn read_object(object: &String)->Result<String, GitrError>{
    let path = parse_object_hash(object)?;
    let bytes = deflate_file(path.clone())?;

    let mut object_data: Vec<u8> = Vec::new();

    for byte in bytes {
        let byte = match byte {
            Ok(byte) => byte,
            Err(_) => return Err(GitrError::FileReadError(path)),
        };
        object_data.push(byte);
    }

    let first_byte = object_data[0];


    let mut object_data_str = String::new();
    for byte in object_data.clone() {
        if byte == 0 {
            break;
        }
        object_data_str.push(byte as char);
    }



    if first_byte as char == 't' {
        let tree_data = match read_tree_file(object_data) {
            Ok(data) => data,
            Err(_) => return Err(GitrError::FileReadError(path)),
        };
        
        return Ok(tree_data);
    }

    if first_byte as char == 'b' || first_byte as char == 'c' {
        let mut buffer = String::new();
        for byte in object_data {
            buffer.push(byte as char);
        }
        return Ok(buffer);
    }


    Err(GitrError::FileReadError("No se pudo leer el objeto, bytes invalidos".to_string()))
}

pub fn read_tree_file(data: Vec<u8>) -> Result<String, GitrError>{
    let mut header_buffer = String::new();
    let mut data_starting_index = 0;
    for byte in data.clone() {
        if byte == 0 {
            data_starting_index += 1;
            break;
        }
        header_buffer.push(byte as char);
        data_starting_index += 1;
    }

    let mut entries_buffer = String::new();
    let mut convert_to_hexa = false;
    let mut hexa_iters = 0;

    for byte in data[data_starting_index..].iter() {

        if hexa_iters == 20 {
            convert_to_hexa = false;
            hexa_iters = 0;
            entries_buffer.push('\n');
        }

        if *byte == 0 && !convert_to_hexa{
            entries_buffer.push('\0');
            convert_to_hexa = true;
            continue;
        }

        if convert_to_hexa {
            hexa_iters+=1;
            entries_buffer.push_str(&format!("{:02x}", byte));
        }
        else {
            entries_buffer.push(*byte as char);
        }


       // buffer.push(*byte as char);
    }


    // 08deed466789dfea8937d0bdda2f6e81a615f25a
    Ok(header_buffer + "\0" + &entries_buffer)
}


// ***writing***

/// A diferencia de write_file, esta funcion recibe un vector de bytes
/// como data, y lo escribe en el archivo de path.
pub fn write_compressed_data(path: &str, data: &[u8]) -> Result<(), GitrError>{
    let log_msg = format!("writing data to: {}", path);
    logger::log_file_operation(log_msg)?;
    match File::create(path) {
        Ok(file) => file,
        Err(_) => return Err(GitrError::FileCreationError(path.to_string())),
    };
    
    match fs::write(path, data) {
        Ok(_) => Ok(()),
        Err(_) => Err(GitrError::FileCreationError(path.to_string()))
    }
    
}

pub fn get_remote() -> Result<String, GitrError> {
    let repo = get_current_repo()?;
    let path = repo + "/gitr/" + "remote";
    let remote = read_file(path)?;
    Ok(remote)
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

// ***others***

fn deflate_file(path: String) -> Result<Bytes<ZlibDecoder<File>>, GitrError> {
    let file = match File::open(&path) {
        Ok(file) => file,
        Err(_) => return Err(GitrError::FileReadError(path.to_string())),
    };
    let decoder = ZlibDecoder::new(file);
   // let mut buffer = String::new();

    let bytes = decoder.bytes();
    Ok(bytes)
}

fn parse_object_hash(object: &String) -> Result<String, GitrError>{
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
    Ok(path)
}





/***************************
 *************************** 
 *      GIT FILES
 **************************
 **************************/


pub fn init_repository(name: &String) ->  Result<(),GitrError>{
        create_directory(name)?;
        create_directory(&(name.clone() + "/gitr"))?;
        create_directory(&(name.clone() + "/gitr/objects"))?;
        create_directory(&(name.clone() + "/gitr/refs"))?;
        create_directory(&(name.clone() + "/gitr/refs/heads"))?;
        create_directory(&(name.clone() + "/gitr/refs/remotes"))?;
        create_directory(&(name.clone() + "/gitr/refs/remotes/daemon"))?;
        write_file(name.clone() + "/gitr/HEAD", "ref: refs/heads/master".to_string())?;
        write_file(name.clone() + "/gitr/index", "".to_string())?;

    Ok(())
}

pub fn get_current_repo() -> Result<String, GitrError>{
    let current_repo = read_file(".head_repo".to_string())?;
    Ok(current_repo)
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
    write_compressed_data(dir.as_str(), compressed_index.as_slice())?;
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


pub fn get_branches()-> Result<Vec<String>, GitrError>{
    let mut branches: Vec<String> = Vec::new();
    let repo = get_current_repo()?;
    let dir = repo + "/gitr/refs/heads";
    let paths = match fs::read_dir(dir.clone()) {
        Ok(paths) => paths,
        Err(_) => return Err(GitrError::FileReadError(dir)),
    };
    for path in paths {
        let path = match path {
            Ok(path) => path,
            Err(_) => return Err(GitrError::FileReadError(dir)),
        };
        let path = path.path();
        let path = path.to_str();
        let path = match path{
            Some(path) => path,
            None => return Err(GitrError::FileReadError(dir)),
        };
        let path = path.split('/').collect::<Vec<&str>>();
        let path = path[path.len()-1];
        branches.push(path.to_string());
    }
    Ok(branches)
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


pub fn get_commit(branch:String)->Result<String, GitrError>{
    let repo = get_current_repo()?;
    let path = format!("{}/gitr/refs/heads/{}",repo, branch);
    let commit = read_file(path.clone())?;
    Ok(commit)
}

pub fn create_tree(path: String, hash: String) -> Result<(), GitrError> {

    file_manager::create_directory(&path)?;

    let tree_raw_data = read_object(&hash)?;


    let raw_data = match tree_raw_data.split_once('\0') {
        Some((_, raw_data)) => raw_data,
        None => {
            println!("Error: invalid object type");
            return Ok(())
        }
    };
    
    for entry in raw_data.split('\n') {
        let object = entry.split(' ').collect::<Vec<&str>>()[0];
        if object == "100644"{ //blob
           
            let path_completo = path.clone() + "/" + &parse_blob_path(entry.to_string().clone());
            let hash = parse_blob_hash(entry.to_string().clone());

            create_blob(path_completo, hash)?;

        } else { //tree
            let _new_path_hash = entry.split(' ').collect::<Vec<&str>>()[1];
            let new_path = _new_path_hash.split('\0').collect::<Vec<&str>>()[0]; 
            let hash = _new_path_hash.split('\0').collect::<Vec<&str>>()[1];
            //println!("{}/{}", folder_path.clone(), new_path);
            create_tree(path.clone() + "/" + new_path, hash.to_string())?;
        }
    }

    Ok(())
}


fn parse_blob_hash(blob_entry: String) -> String {
    let _new_path_hash = blob_entry.split(' ').collect::<Vec<&str>>()[1];
    let hash = _new_path_hash.split('\0').collect::<Vec<&str>>()[1];
    hash.to_string()
}

fn parse_blob_path(blob_entry: String) -> String {
    let _new_path_hash = blob_entry.split(' ').collect::<Vec<&str>>()[1];
    let new_path = _new_path_hash.split('\0').collect::<Vec<&str>>()[0];
    new_path.to_string()
}


// archivo
// repo/carpeta/archivo

pub fn create_blob(path: String, hash: String) -> Result<(), GitrError> {

    let new_blob = read_object(&(hash.to_string()))?;
    
    let new_blob_only_data = new_blob.split('\0').collect::<Vec<&str>>()[1];
    add_to_index(&path, &hash)?;

    write_file(path.to_string(), new_blob_only_data.to_string())?;
    Ok(())
}

pub fn update_working_directory(commit: String)-> Result<(), GitrError>{
    delete_all_files()?;
    let main_tree = get_main_tree(commit)?;
    let tree = read_object(&main_tree)?;

    let raw_data = match tree.split_once('\0') {
        Some((_, raw_data)) => raw_data,
        None => {
            println!("Error: invalid object type");
            return Ok(())
        }
    };

    let repo = get_current_repo()? + "/";


    for entry in raw_data.split('\n'){
        let object: &str = entry.split(' ').collect::<Vec<&str>>()[0];
        if object == "40000"{
            let _new_path_hash = entry.split(' ').collect::<Vec<&str>>()[1];
            let new_path = repo.clone() + _new_path_hash.split('\0').collect::<Vec<&str>>()[0];
            let hash = _new_path_hash.split('\0').collect::<Vec<&str>>()[1];
            create_tree(new_path.to_string(), hash.to_string())?;
        } else{
            let path_completo = repo.clone() + parse_blob_path(entry.to_string().clone()).as_str();
            let hash = parse_blob_hash(entry.to_string().clone());

            create_blob(path_completo, hash)?;
        }
    }
    Ok(())
}

pub fn get_main_tree(commit:String)->Result<String, GitrError>{
    let commit = read_object(&commit)?;
    let commit = commit.split('\n').collect::<Vec<&str>>();
    let tree_base = commit[0].split('\0').collect::<Vec<&str>>()[1];
    let tree_hash_str = tree_base.split(' ').collect::<Vec<&str>>()[1];
    Ok(tree_hash_str.to_string())
}

pub fn get_parent_commit(commit: String)->Result<String, GitrError>{
    let commit = read_object(&commit)?;
    let commit = commit.split('\n').collect::<Vec<&str>>();
    if commit[1].split(' ').collect::<Vec<&str>>()[0] != "parent"{
        return Ok("None".to_string());
    }
    let parent = commit[1].split(' ').collect::<Vec<&str>>()[1];
    Ok(parent.to_string())
}

pub fn get_commit_author(commit: String)->Result<String, GitrError>{
    let commit = read_object(&commit)?;
    let commit = commit.split('\n').collect::<Vec<&str>>();
    let mut idx = 2;
    if commit[1].split(' ').collect::<Vec<&str>>()[0] != "parent"{
        idx -= 1;
    }
    let author = commit[idx].split(' ').collect::<Vec<&str>>()[1];
    Ok(author.to_string())
}

pub fn get_commit_date(commit: String)->Result<String, GitrError>{
    let commit = read_object(&commit)?;
    let commit = commit.split('\n').collect::<Vec<&str>>();
    let mut idx = 2;
    if commit[1].split(' ').collect::<Vec<&str>>()[0] != "parent"{
        idx -= 1;
    }
    let timestamp = commit[idx].split(' ').collect::<Vec<&str>>()[3];
    let timestamp_parsed = match timestamp.parse::<i64>(){
        Ok(timestamp) => timestamp,
        Err(_) => return Err(GitrError::TimeError),
    };
    let dt = Utc.timestamp_opt(timestamp_parsed, 0);
    let dt = match dt.single(){
        Some(dt) => dt,
        None => return Err(GitrError::TimeError),
    };

    let offset = FixedOffset::east_opt(-3 * 3600);
    let offset = match offset{
        Some(offset) => offset,
        None => return Err(GitrError::TimeError),
    };
    let dt = dt.with_timezone(&offset);

    let date = dt.format("%a %b %d %H:%M:%S %Y %z").to_string();
    Ok(date)
}

pub fn get_commit_message(commit: String)->Result<String, GitrError>{
    let commit = read_object(&commit)?;
    let commit = commit.split('\n').collect::<Vec<&str>>();
    let mut idx = 5;
    if commit[1].split(' ').collect::<Vec<&str>>()[0] != "parent"{
        idx -= 1;
    }
    let message = commit[idx..].join("\n");
    Ok(message)
}

pub fn update_current_repo(dir_name: &String) -> Result<(), GitrError> {
    write_file(".head_repo".to_string(), dir_name.to_string())?;

    Ok(())
}

/// Devuelve vector con los ids de los commits en los heads activos
pub fn get_heads_ids() -> Result<Vec<String>, GitrError> {
    let mut branches: Vec<String> = Vec::new();
    let repo = get_current_repo()?;
    let dir = repo + "/gitr/refs/heads";
    let paths = match fs::read_dir(dir.clone()) {
        Ok(paths) => paths,
        Err(_) => return Err(GitrError::FileReadError(dir)),
    };
    for path in paths {
        let path = match path {
            Ok(path) => path,
            Err(_) => return Err(GitrError::FileReadError(dir)),
        };
        let path = path.path();
        let path = path.to_str();
        let path = match path{
            Some(path) => path,
            None => return Err(GitrError::FileReadError(dir)),
        };
        let content = read_file(path.to_string())?;
        branches.push(content);
    }
    Ok(branches)
}

pub fn commit_log(quantity: String)-> Result<String, GitrError>{
    let mut res:String = "".to_owned();
    let mut current_commit = get_current_commit()?;
    let limit = match quantity.parse::<i32>(){
        Ok(quantity) => quantity,
        Err(_) => return Err(GitrError::InvalidArgumentError(quantity, "log <quantity>".to_string())),
    };
    let mut counter = 0;
    loop{
        counter += 1;
        let format_commit = format!("commit: {}\n", current_commit);
        res.push_str(&format_commit);
        let parent = get_parent_commit(current_commit.clone())?;
        let date = get_commit_date(current_commit.clone())?;
        let author = get_commit_author(current_commit.clone())?;
        let message = get_commit_message(current_commit.clone())?;
        res.push_str(&format!("Author: {}\n", author));
        res.push_str(&format!("Date: {}\n", date));
        res.push_str(&format!("\t{}\n\n", message));
        if parent == "None" || counter == limit{
            break;
        }
        current_commit = parent;
    }

    Ok(res.to_string())
}

pub fn get_repos() -> Vec<String> {
    let mut repos: Vec<String> = Vec::new();
    if let Ok(entries) = fs::read_dir("./") {
        for entry in entries.flatten() {
            if entry.file_name() == "gitr" || 
                entry.file_name() == "src" ||
                entry.file_name() == "tests" ||
                entry.file_name() == "target" {
                continue;
            }
            if entry.file_type().unwrap().is_dir() {
                repos.push(entry.path().display().to_string()[2..].to_string());
            }
        }
    }
    repos
}

pub fn remove_file(path: String)-> Result<(), GitrError> {
    match fs::remove_file(path.clone()) {
        Ok(_) => Ok(()),
        Err(_) => Err(GitrError::FileDeleteError(path)),
    }
}
pub fn get_all_objects() -> Result<Vec<String>,GitrError> {
    let mut objects: Vec<String> = Vec::new();
    let repo = get_current_repo()?;
    let dir = repo + "/gitr/objects";
    let dir_reader = match fs::read_dir(dir.clone()) {
        Ok(l) => l,
        Err(_) => return Err(GitrError::FileReadError(dir)),
    };
    for carpeta_rs in dir_reader {
        let carpeta = match carpeta_rs {
            Ok(path) => path,
            Err(_) => return Err(GitrError::FileReadError(dir)),
        };
        let f = carpeta.file_name();
        let dir_name = f.to_str().unwrap_or("Error");
        if dir_name == "Error" {
            return Err(GitrError::FileReadError(dir));
        }
        let file_reader = match fs::read_dir(dir.clone() + "/" + dir_name.clone()) {
            Ok(l) => l,
            Err(_) => return Err(GitrError::FileReadError(dir)),
        };
        for file in file_reader {
            let file = match file {
                Ok(path) => path,
                Err(_) => return Err(GitrError::FileReadError(dir)),
            };
            let f = file.file_name();
            let file_name = f.to_str().unwrap_or("Error");
            if file_name == "Error" {
                return Err(GitrError::FileReadError(dir));
            }
            let object = dir_name.to_string() + file_name;
            objects.push(object);
        }

    }
    Ok(objects)
}

pub fn get_object(id: String, r_path: String) -> Result<String,GitrError> {
    println!("llega a pedir object{:?}",id);
    let dir_path = format!("{}/objects/{}",r_path.clone(),id.split_at(2).0);
    let mut archivo = match File::open(&format!("{}/{}",dir_path,id.split_at(2).1)) {
        Ok(archivo) => archivo,
        Err(_) => return Err(GitrError::FileReadError(dir_path)),
    }; // si no existe tira error
    let mut contenido: Vec<u8>= Vec::new();
    if let Err(_) = archivo.read_to_end(&mut contenido) {
        return Err(GitrError::FileReadError(dir_path));
    }
    let descomprimido = String::from_utf8_lossy(&decode(&contenido)?).to_string();
    println!("llega a dar object");
    Ok(descomprimido)
}
pub fn get_object_bytes(id: String, r_path: String) -> Result<Vec<u8>,GitrError> {
    let dir_path = format!("{}/objects/{}",r_path.clone(),id.split_at(2).0);
    let mut archivo = match File::open(&format!("{}/{}",dir_path,id.split_at(2).1)) {
        Ok(archivo) => archivo,
        Err(_) => return Err(GitrError::FileReadError(dir_path)),
    }; // si no existe tira error
    let mut contenido: Vec<u8>= Vec::new();
    if let Err(_) = archivo.read_to_end(&mut contenido) {
        return Err(GitrError::FileReadError(dir_path));
    }
    Ok(decode(&contenido)?)
}

pub fn decode(input: &[u8]) -> Result<Vec<u8>, GitrError> {
    let mut decoder = ZlibDecoder::new(input);
    let mut decoded_data = Vec::new();
    if let Err(_) = decoder.read_to_end(&mut decoded_data) {
        return Err(GitrError::CompressionError);
    }
    Ok(decoded_data)
}