use std::collections::HashMap;
use std::hash::Hash;
use std::path::Path;
use crate::file_manager::{get_head, get_main_tree, get_parent_commit,get_current_repo};

use crate::command_utils::{*, self};
use std::net::TcpStream;
use std::io::prelude::*;
use std::{fs, hash};
use std::ops::IndexMut;
use crate::file_manager::{commit_log, update_working_directory, get_current_commit};
use crate::objects::git_object::GitObject::*;
use crate::{objects::blob::Blob, file_manager, gitr_errors::GitrError, git_transport::pack_file::PackFile};
use crate::git_transport::pack_file::read_pack_file;
use crate::{command_utils::*, commands};
use crate::git_transport::ref_discovery;

/***************************
 *************************** 
 *      COMMANDS
 **************************
 **************************/


//Create an empty Gitr repository 
pub fn init(flags: Vec<String>) -> Result<(), GitrError> {
    // init <name-of-new-repo>
    if flags.is_empty() || flags.len() > 1  {
        return Err(GitrError::InvalidArgumentError(flags.join(" "), "init <new_repo_name>".to_string()));
    }
    file_manager::init_repository(&flags[0])?;
    file_manager::update_current_repo(&flags[0])?;
    println!("Initialized empty Gitr repository");
    Ok(())
}

/// Computes the object ID value for an object with the contents of the named file 
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
    println!("{}", blob.get_hash());
    if write {
        blob.save()?;
    }
    Ok(())
}


//Output the contents or other properties such as size, type or delta information of an object 
pub fn cat_file(flags: Vec<String>) -> Result<(),GitrError> {
    //cat-file -p <object-hash>
    //cat-file -t <object-hash>
    //cat-file -s <object-hash>
    if flags.len() != 2 {
        let flags_str = flags.join(" ");
        return Err(GitrError::InvalidArgumentError(flags_str,"cat-file <[-t/-s/-p]> <object hash>".to_string()));
    }
    let (object_hash, res_output, size, object_type) = get_object_properties(flags.clone())?;
    print_cat_file_command(&flags[0].clone(), &object_hash, &object_type, res_output.clone(), &size)?;
    Ok(())
}


//Add file contents to the index
pub fn add(flags: Vec<String>)-> Result<(), GitrError> {
    // add <file-name>
    // add .
    if flags.len() != 1 {
        return Err(GitrError::InvalidArgumentError(flags.join(" "), "add <[file/.]>".to_string()))
    }
    update_index_before_add()?;
    add_files_command(flags[0].clone())?;
    Ok(())
    
}
//Remove files from the index
pub fn rm(flags: Vec<String>)-> Result<(), GitrError> {
    //rm <file>
    if flags.len() != 1 {
        return Err(GitrError::InvalidArgumentError(flags.join(" "), "rm <file>".to_string()))
    }
    let removed = rm_from_index(&flags[0])?;
    if removed{
        println!("rm '{}'", flags[0]);
    }else{
        println!("Error: file not found");
    }
    Ok(())
} 

//Record changes to the repository
pub fn commit(flags: Vec<String>)-> Result<(), GitrError>{
    //commit -m <message-of-commit>
    if flags[0] != "-m" || flags.len() < 2 {
        return Err(GitrError::InvalidArgumentError(flags.join(" "), "commit -m <commit_message>".to_string()))
    }
    if flags[1].starts_with('\"'){
        let message = &flags[1..];
        let message = message.join(" ");
        get_tree_entries(message.to_string())?;
        print_commit_confirmation(message)?;
        return Ok(())
    } else {
        return Err(GitrError::InvalidArgumentError(flags.join(" "), "commit -m \"commit_message\"".to_string()))
    }
}

// Switch branches or restore working tree files
pub fn checkout(flags: Vec<String>)->Result<(), GitrError> {
    if flags.len() != 1 {
        return Err(GitrError::InvalidArgumentError(flags.join(" "), "checkout <branch>".to_string()));
    }
    if !branch_exists(flags[0].clone()){
        return Err(GitrError::BranchNonExistsError(flags[0].clone()));
    }
    let current_commit = file_manager::get_commit(flags[0].clone())?;
    file_manager::update_working_directory(current_commit)?;
    let path_head = format!("refs/heads/{}", flags[0]);
    file_manager::update_head(&path_head)?;
    
    Ok(())
}

//Show commit logs
pub fn log(flags: Vec<String>)->Result<(), GitrError> {
    // log 
    if flags.is_empty() {
       let log_res = commit_log("-1".to_string())?;
       print!("{}", log_res);
    }
    if flags.len() == 2 && flags[0] == "-n" && flags[1].parse::<usize>().is_ok(){
        let log_res = commit_log(flags[1].to_string())?;
        print!("{}", log_res);
    }
    Ok(())
}

pub fn push(_flags: Vec<String>) {
    println!("push");
}

// List, create, or delete branches
pub fn branch(flags: Vec<String>)->Result<(), GitrError>{
    //branch -m <origin_branch> <destination-branch>
    //branch
    //branch -d <branch-to-delete>
    //branch -l
    //branch <new-branch-name>
    if flags.is_empty() || (flags.len() == 1 && flags[0] == "-l") || (flags.len() == 1 && flags[0] == "--list"){
        print_branches()?;
        return Ok(())
    }
    commit_existing()?;
    if flags.len() == 2 && flags[0] == "-d"{
        branch_delete_flag(flags[1].clone())?;
    }
    if flags.len() == 3 && flags[0] == "-m"{
        branch_move_flag(flags[1].clone(), flags[2].clone())?;
    }
    if flags.len() == 1 && flags[0] != "-l" && flags[0] != "--list"{
        branch_newbranch_flag(flags[0].clone())?;
    }
    Ok(())
}

pub fn ls_files(flags: Vec<String>) -> Result<(), GitrError>{
    //ls-files --stage
    if flags.len() != 1 {
        return Err(GitrError::InvalidArgumentError(flags.join(" "),"ls-files --stage".to_string() ))
    }

    if flags[0] == "--stage"{
        let res_output = file_manager::read_index()?;
        println!("{}", res_output);
    }
    Ok(())
}

pub fn clone(flags: Vec<String>)->Result<(),GitrError>{
    let address = flags[0].clone();
    let nombre_repo = flags[1].clone();

    init(vec![nombre_repo.clone()])?;

    let mut socket = clone_connect_to_server(address)?;
    // println!("clone():Servidor conectado.");
    clone_send_git_upload_pack(&mut socket)?;
    // println!("clone():Envié upload-pack");
    let ref_disc = clone_read_reference_discovery(&mut socket)?;
    let references = ref_discovery::discover_references(ref_disc)?;

    let repo = file_manager::get_current_repo()?;
    
    for reference in &references[1..]{
        let path_str = repo.clone() + "/gitr/"+ &reference.1.clone(); //ref path
        if references[0].0 == reference.0{
            file_manager::update_head(&reference.1.clone())?; //actualizo el head
        }
        let into_hash = reference.0.clone(); //hash a escribir en el archivo
        file_manager::write_file(path_str, into_hash)?; //escribo el hash en el archivo
    }

    // println!("clone():Referencias ={:?}=", references);
    let want_message = ref_discovery::assemble_want_message(&references,Vec::new())?;
    // println!("clone():want {:?}", want_message);

    write_socket(&mut socket, want_message.as_bytes())?;

    let mut buffer = [0;1024];
    match socket.read(&mut buffer){
        Ok(a)=>a,
        Err(e)=>return Err(GitrError::SocketError("clone".into(), e.to_string()))
    };
    
    print!("clone(): recepeción de packfile:");
    read_socket(&mut socket, &mut buffer)?;

    let pack_file_struct = PackFile::new_from_server_packfile(&mut buffer)?;

    for object in pack_file_struct.objects.iter(){
        match object{
            Blob(blob) => blob.save()?,
            Commit(commit) => commit.save()?,
            Tree(tree) => tree.save()?,
        }
    }
    update_working_directory(get_current_commit()?)?;
    Ok(())
}

// Show the working tree status
pub fn status(flags: Vec<String>) -> Result<(), GitrError>{
    command_utils::status_print_current_branch()?;
    let working_dir_hashmap = get_working_dir_hashmap()?;
    let (index_hashmap, hayindex) = get_index_hashmap()?;
    let current_commit_hashmap = get_current_commit_hashmap()?;

    let mut to_be_commited = Vec::new();
    let mut not_staged = Vec::new();
    let mut untracked_files = Vec::new();

    // compare to working dir
    for (path, hash) in working_dir_hashmap.clone().into_iter() {
        if !index_hashmap.contains_key(path.as_str()) && !current_commit_hashmap.contains_key(path.as_str()) {
            untracked_files.push(path.clone());
        }
        if current_commit_hashmap.contains_key(path.clone().as_str()){
            if let Some(commit_hash) = current_commit_hashmap.get(path.as_str()) {
                if &hash != commit_hash {
                    if !index_hashmap.contains_key(&path) {
                        not_staged.push(path.clone( ));
                    }
                }
            };
        }
        if index_hashmap.contains_key(path.as_str()){
            if let Some(index_hash) = index_hashmap.get(path.as_str()) {
                if &hash != index_hash {
                    not_staged.push(path);
                }
            };
        }
    }
    // compare to index
    for (path, hash) in index_hashmap.clone().into_iter() {
        if !current_commit_hashmap.contains_key(path.as_str()) {
            to_be_commited.push(path);
        }
        else {
            if let Some(commit_hash) = current_commit_hashmap.get(path.as_str()) {
                if hash != *commit_hash  &&
                !not_staged.contains(&path)
                {
                    to_be_commited.push(path);
                }
            }
        }
    }
    status_print_to_be_comited(&to_be_commited);
    status_print_not_staged(&not_staged);
    status_print_untracked(&untracked_files, hayindex);
    if to_be_commited.is_empty() && not_staged.is_empty() && untracked_files.is_empty() {
        println!("nothing to commit, working tree clean");
    }
    Ok(())
}


pub fn fetch(_flags: Vec<String>) {
    println!("fetch");
}

pub fn merge(_flags: Vec<String>) -> Result<(), GitrError>{
    if _flags.len() == 0{
        return Err(GitrError::InvalidArgumentError(_flags.join(" "), "merge <branch-name>".to_string()))
    }

    let branch_name = _flags[0].clone();
    let origin_name = file_manager::get_head()?.split('/').collect::<Vec<&str>>()[2].to_string();

    let branch_commits = command_utils::branch_commits_list(branch_name.clone())?;
    let origin_commits = command_utils::branch_commits_list(origin_name)?;

    for commit in branch_commits.clone() {
        if origin_commits.contains(&commit) {
            if commit == origin_commits[origin_commits.len() -1]{
                // fast-forward merge (caso facil)
 
                println!("Updating {}..{}" ,&origin_commits[origin_commits.len() -1][..7], &branch_commits[branch_commits.len() -1][..7]);
                println!("Fast-forward");

                command_utils::fast_forward_merge(branch_name)?;
                break;
            }
            // three way merge (caso dificil)
            //command_utils::three_way_merge();
        }
    }
    Ok(())
}

pub fn remote(_flags: Vec<String>) {
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
    let msj = format!("git-upload-pack /{}\0host={}\0","mi-repo", remote);
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
    println!("\n\nreferencias: {:?}\n\n",hash_n_references);

    let want_message = ref_discovery::assemble_want_message(&hash_n_references,file_manager::get_heads_ids()?)?;
    
    match stream.write(want_message.as_bytes()) {
        Ok(_) => (),
        Err(e) => {
            println!("Error: {}", e);
            return Ok(())
        }
    };
    println!("\n\nwant message{}\n\n",want_message);

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
        
        Err(e) => {
            println!("Error: {}", e);
            return Ok(())
        },
        _ => ()
    }
    let pack_file_struct = PackFile::new_from_server_packfile(&mut buffer)?;
    for object in pack_file_struct.objects.iter(){
        match object{
            Blob(blob) => blob.save()?,
            Commit(commit) => commit.save()?,
            Tree(tree) => tree.save()?,
        }
    }
    update_working_directory(get_current_commit()?)?;
    println!("pull successfull");
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

