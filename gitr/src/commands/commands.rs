use std::net::TcpStream;
// use std::fmt::Result;
use std::{io::prelude::*, error::Error};

use chrono::format;

use crate::{objects::blob::Blob, file_manager, gitr_errors::GitrError, git_transport::pack_file::read_pack_file};
use crate::file_manager::print_commit_log;
use crate::command_utils::*;

use crate::git_transport::ref_discovery;

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


pub fn clone(flags: Vec<String>)->Result<(),GitrError>{
    let address = flags[0].clone();
    let mut socket = clone_connect_to_server(address)?;
    // println!("clone():Servidor conectado.");
    clone_send_git_upload_pack(&mut socket)?;
    // println!("clone():Envié upload-pack");
    let ref_disc = clone_read_reference_discovery(&mut socket)?;
    let references = ref_discovery::discover_references(ref_disc)?;
    // println!("clone():Referencias ={:?}=", references);
    let want_message = ref_discovery::assemble_want_message(&references,Vec::new())?;
    // println!("clone():want {:?}", want_message);

    match socket.write(want_message.as_bytes()) {
        Ok(_) => (),
        Err(e) => {
            println!("Error: {}", e);
            return Ok(())
        }
    }

    let mut buffer = [0;1024];
    match socket.read(&mut buffer) {
        Err(e) => {
            println!("Error: {}", e);
            return Ok(())
        }
        _ => ()
    }
    print!("clone(): recepeción de packfile:");

    let objects = read_pack_file(&mut buffer);
    Ok(())
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

pub fn pull(flags: Vec<String>) -> Result<(), GitrError> {
    if !flags.is_empty(){
        return Err(GitrError::InvalidArgumentError(flags.join(" "), "pull <no-args>".to_string()));
    }
    // "003agit-upload-pack /schacon/gitbook.git\0host=example.com\0"

    // ########## HANDSHAKE ##########
    let repo = file_manager::get_current_repo()?;
    let remote = file_manager::get_remote()?;
    let msj = format!("git-upload-pack /{}\0host={}\0",repo, remote);
    let msj = format!("{:04x}{}", msj.len() + 4, msj);
    let mut stream = match TcpStream::connect(remote) {
        Ok(socket) => socket,
        Err(e) => {
            println!("Error: {}", e);
            return Ok(())
        }
    };
    match stream.write(msj.as_bytes()) {
        Ok(_) => (),
        Err(e) => {
            println!("Error: {}", e);
            return Ok(())
        }
    };
    
    //  ########## REFERENCE DISCOVERY ##########
    let mut buffer = [0;1024];
    let mut ref_disc = String::new();
    loop {
        match stream.read(&mut buffer) {
            Ok(n) => {
                let bytes = &buffer[..n];
                let s = String::from_utf8_lossy(bytes);
                ref_disc.push_str(&s);
                if n < 1024 {
                    break;
                }
            },
            Err(e) => {
                println!("Error: {}", e);
                return Ok(())
            }
        }
    }
    let hash_n_references = ref_discovery::discover_references(ref_disc)?;
    let want_message = ref_discovery::assemble_want_message(&hash_n_references,file_manager::get_heads_ids()?)?;
    match stream.write(want_message.as_bytes()) {
        Ok(_) => (),
        Err(e) => {
            println!("Error: {}", e);
            return Ok(())
        }
    };
    match stream.read(&mut buffer) { // Leo si huvo error
        Ok(_n) => {if String::from_utf8_lossy(&buffer).contains("Error") {
            println!("Error: {}", String::from_utf8_lossy(&buffer));
            return Ok(())
        }},
        Err(e) => {
            println!("Error: {}", e);
            return Ok(())
        }
        
    }
    
    match stream.read(&mut buffer) { // Leo el packfile
        Ok(_n) => {
            let objects = read_pack_file(&mut buffer);
            println!("objects: {:?}", objects);
        },
        Err(e) => {
            println!("Error: {}", e);
            return Ok(())
        }
        
    }
    println!("pull successfull");
    Ok(())
}

pub fn push(flags: Vec<String>) {
    println!("push");
}

pub fn branch(flags: Vec<String>)->Result<(), GitrError>{
    if flags.is_empty() || (flags.len() == 1 && flags[0] == "-l") || (flags.len() == 1 && flags[0] == "--list"){
        match print_branches() {
            Ok(()) => (),
            Err(e) => return Err(GitrError::InvalidArgumentError(flags.join(" "), "TODO: escribir como se usa branch aca".into()))
        };
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
        match file_manager::move_branch(old_path.clone(), new_path.clone()) {
            Ok(()) => (),
            Err(e) => return Err(GitrError::InvalidArgumentError(flags.join(" "), "TODO: escribir como se usa branch aca".into()))
        };
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
        let res_output = file_manager::read_index()?;
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

#[cfg(test)]
mod tests{

    use super::*;
    #[test]
    fn test00_clone_from_daemon(){
        let mut flags = vec![];
        flags.push("localhost:9418".to_string());
        assert!(clone(flags).is_ok());
    }
}