use std::error::Error;

use crate::{objects::blob::Blob, file_manager::{self, print_commit_log}, gitr_errors::GitrError};
use crate::command_utils::*;


/*
    NOTA: Puede que no todos los comandos requieran de flags,
    si ya esta hecha la funcion y no se uso, se puede borrar
    (y hay que modificar el llamado desde handler.rs tambien)
*/


/// Computes the object ID value for an object with the contents of the named file 
/// When <type> is not specified, it defaults to "blob".
pub fn hash_object(flags: Vec<String>) -> Result<(), GitrError>{
    // hash-object -w <file>
    // hash-object <file>

    if flags.len() != 1 && flags.len() != 2 {
        return Err(GitrError::InvalidArgumentError(flags.join(" "), "hash-object [<-w>] <file>".to_string()));
    }

    let mut file_path = String::new();
    let mut write = false;

    if flags.len() == 1 {
        file_path = flags[0].clone();
    }

    if flags.len() == 2 && flags[0] == "-w" {
        file_path = flags[1].clone();
        write = true;
    }
    
    file_path = file_manager::get_current_repo()?.to_string() + "/" + &file_path;

    let raw_data = file_manager::read_file(file_path)?;
    
    let blob = Blob::new(raw_data)?;
    // cuando haga falta, aca con un switch podemos 
    // crear tree o commit tambien
    
    println!("{}", blob.get_hash());
    println!();

    if write {
        blob.save()?;
    }

    Ok(())
}

pub fn cat_file(flags: Vec<String>) -> Result<(),GitrError> {
    if flags.len() != 2 {
        let flags_str = flags.join(" ");
        return Err(GitrError::InvalidArgumentError(flags_str,"cat-file <[-t/-s/-p]> <object hash>".to_string()));
    }
    let res_output = file_manager::read_object(&flags[1])?;
    println!("data cruda: {:?}", res_output);
    let object_type = res_output.split(' ').collect::<Vec<&str>>()[0];
    let _size = res_output.split(' ').collect::<Vec<&str>>()[1];
    let size = _size.split('\0').collect::<Vec<&str>>()[0];


    if flags[0] == "-t"{
        println!("{}", object_type);
    }
    if flags[0] == "-s"{
        println!("{}", size);
    }
    if flags[0] == "-p"{
        let raw_data_index = match res_output.find('\0') {
            Some(index) => index,
            None => {
                println!("Error: invalid object type");
                return Ok(())
            }
        };

        let raw_data = &res_output[(raw_data_index + 1)..];
        match object_type {
            "blob" => print_blob_data(raw_data),
            "tree" => print_tree_data(raw_data),
            "commit" => println!("{}", res_output.split('\0').collect::<Vec<&str>>()[1]),
            _ => println!("Error: invalid object type"),
        }
    }
    
    Ok(())
}

pub fn init(flags: Vec<String>) -> Result<(), GitrError> {
    if flags.is_empty() || flags.len() > 1  {
        return Err(GitrError::InvalidArgumentError(flags.join(" "), "init <new_repo_name>".to_string()));
    }

    file_manager::init_repository(&flags[0])?;
    file_manager::update_current_repo(&flags[0])?;
    println!("Initialized empty Gitr repository");
    Ok(())
}

pub fn status(flags: Vec<String>) {
    println!("status");
}

// pub fn create_blob_from_file(file_path: &String) -> Result<(), Box<dyn Error>> {
//     let raw_data = file_manager::read_file(file_path.to_string())?;
//     let blob = Blob::new(raw_data)?;
//     blob.save()?;
//     let hash = blob.get_hash();
//     file_manager::add_to_index(file_path, &hash)?;
//     Ok(())
// }


fn save_and_add_blob_to_index(file_path: String) -> Result<(), GitrError> {
    let raw_data = file_manager::read_file(file_path.clone())?;
    let blob = Blob::new(raw_data)?;
    blob.save()?;
    let hash = blob.get_hash();
    file_manager::add_to_index(&file_path, &hash)?;
    Ok(())
}

pub fn add(flags: Vec<String>)-> Result<(), GitrError> {
    if flags.len() != 1 {
        return Err(GitrError::InvalidArgumentError(flags.join(" "), "add <[file/.]>".to_string()))
    }
    // check if flags[0] is an existing file
    let file_path = &flags[0];

    let repo = file_manager::get_current_repo()?;

    if file_path == "."{
        let files = visit_dirs(std::path::Path::new(&repo));
        for file in files{
            if file.contains("gitr"){
                continue
            }
            save_and_add_blob_to_index(file.clone())?;
        }
    }else{
        let full_file_path = repo + "/" + file_path;
        save_and_add_blob_to_index(full_file_path)?;
    }
    Ok(())
    
}

pub fn rm(flags: Vec<String>)-> Result<(), GitrError> {
    let mut removed:bool = false;
    if flags.len() != 1 {
        return Err(GitrError::InvalidArgumentError(flags.join(" "), "rm <file>".to_string()))
    }
    let mut index = file_manager::read_index()?;
    index += "\n";
    let current_repo = file_manager::get_current_repo()?;
    let file_to_rm_path = format!("{}/{}", current_repo, flags[0]);
    for line in index.lines(){
        let attributes = line.split(' ').collect::<Vec<&str>>();
        if attributes[3] == file_to_rm_path{
            let complete_line = format!("{}\n", line);
            index = index.replace(&complete_line, "");
            let res = index.trim_end().to_string();
            removed = true;
            let compressed_index = flate2compress(res)?;
            let _ = file_manager::write_compressed_data(&(current_repo +"/gitr/index"), compressed_index.as_slice());
            break
        }
    }
    if removed{
        println!("rm '{}'", flags[0]);
    }else{
        println!("Error: file not found");
    }
    Ok(())
  
} 

// estamos haciendo un tree de mas
pub fn commit(flags: Vec<String>)-> Result<(), GitrError>{
    if flags[0] != "-m" || flags.len() < 2 {
        return Err(GitrError::InvalidArgumentError(flags.join(" "), "commit -m <commit_message>".to_string()))
    }
    
    let message = &flags[1];
    println!("message: {:?}", message);
    if flags[1].starts_with('\"'){
        let message = &flags[1..];
        let message = message.join(" ");
        get_tree_entries(message.to_string())?;
        return Ok(())
    }
    get_tree_entries(message.to_string())?;
    
    Ok(())
}

pub fn checkout(flags: Vec<String>)->Result<(), GitrError> {
    if flags.len() != 1 {
        return Err(GitrError::InvalidArgumentError(flags.join(" "), "checkout <branch>".to_string()));
    }

    if !branch_exists(flags[0].clone()){
        println!("error: pathspec '{}' did not match any file(s) known to git.", flags[0]);
        return Ok(())
    }
    let current_commit = file_manager::get_commit(flags[0].clone())?;
    file_manager::update_working_directory(current_commit)?;
    let path_head = format!("refs/heads/{}", flags[0]);
    file_manager::update_head(&path_head)?;
    
    Ok(())
}

pub fn log(flags: Vec<String>)->Result<(), GitrError> {
    if flags.is_empty() {
       print_commit_log("-1".to_string())?;
    }
    if flags.len() == 2 && flags[0] == "-n" && flags[1].parse::<usize>().is_ok(){
        print_commit_log(flags[1].to_string())?;
    }
    Ok(())
}

pub fn clone(flags: Vec<String>) {
    println!("clone");
}

pub fn fetch(flags: Vec<String>) {
    println!("fetch");
}

pub fn merge(flags: Vec<String>) {
    println!("merge");
}

pub fn remote(flags: Vec<String>) {
    println!("remote");
}

pub fn pull(flags: Vec<String>) {
    println!("pull");
}

pub fn push(flags: Vec<String>) {
    println!("push");
}

pub fn branch(flags: Vec<String>)->Result<(), Box<dyn Error>>{
    if flags.is_empty() || (flags.len() == 1 && flags[0] == "-l") || (flags.len() == 1 && flags[0] == "--list"){
        print_branches()?;
    }
    if flags.len() == 2 && flags[0] == "-d"{
        // falta chequear si el branch está al día, xq sino se usa -D
        if !branch_exists(flags[1].clone()){
            println!("error: branch '{}' not found.", flags[1]);
            return Ok(())
        }
        let _ = file_manager::delete_branch(flags[1].clone(), false);
        return Ok(())
    }
    if flags.len() == 2 && flags[0] == "-D"{
        if !branch_exists(flags[1].clone()){
            println!("error: branch '{}' not found.", flags[1]);
            return Ok(())
        }
        let _ = file_manager::delete_branch(flags[1].clone(), false);
        return Ok(())
    }
    if flags.len() == 3 && flags[0] == "-m"{
        if !branch_exists(flags[1].clone()){
            println!("error: branch '{}' not found.", flags[1]);
            return Ok(())
        }
        if branch_exists(flags[2].clone()){
            println!("error: a branch named '{}' already exists.", flags[2]);
            return Ok(())
        }
        let repo = file_manager::get_current_repo()?;
        let old_path = format!("{}/gitr/refs/heads/{}", repo, flags[1]);
        let new_path = format!("{}/gitr/refs/heads/{}", repo, flags[2]);
        file_manager::move_branch(old_path.clone(), new_path.clone())?;
        file_manager::update_head(&new_path)?;
        return Ok(())

    }
    if flags.len() == 1 && flags[0] != "-l" && flags[0] != "--list"{
        if branch_exists(flags[0].clone()){
            println!("fatal: A branch named '{}' already exists.", flags[0]);
            return Ok(())
        }
        let current_commit = file_manager::get_current_commit()?;
        let repo = file_manager::get_current_repo()?;
        file_manager::write_file(format!("{}/gitr/refs/heads/{}", repo, flags[0]), current_commit)?;
    }
    Ok(())
}

pub fn ls_files(flags: Vec<String>) -> Result<(), GitrError>{
    if flags.len() != 1 {
        return Err(GitrError::InvalidArgumentError(flags.join(" "),"ls-files --stage".to_string() ))
    }

    if flags[0] == "--stage"{
        let res_output = file_manager::read_index().unwrap();
        println!("{}", res_output);
    }
    Ok(())
}

pub fn list_repos() {
    println!("{:?}", file_manager::get_repos());
}

pub fn go_to_repo(flags: Vec<String>) -> Result<(), GitrError>{
    if flags.len() != 1 {
        return Err(GitrError::InvalidArgumentError(flags.join(" "), "go-to-repo <repo>".to_string()));   
    }

    let new_repo = flags[0].clone();
    let existing_repos = file_manager::get_repos();

    if existing_repos.contains(&new_repo) {
        file_manager::update_current_repo(&new_repo)?;
    }
    else {
        println!("Error: repository '{}' does not exist", new_repo);
    }
    Ok(())
}

pub fn print_current_repo() -> Result<(), GitrError> {
    let repo = file_manager::get_current_repo()?;
    println!("working on repo: {}", repo);

    Ok(())
}

