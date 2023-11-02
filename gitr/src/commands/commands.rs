use std::collections::HashMap;
use std::hash::Hash;
use std::path::Path;

use crate::{objects::blob::Blob, file_manager, gitr_errors::GitrError, git_transport::pack_file::read_pack_file};
use crate::file_manager::{print_commit_log, get_head, get_main_tree};
use crate::command_utils::{*, self};

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

    let data_requested = &flags[0];
    let object_hash = &flags[1];

    let res_output = file_manager::read_object(object_hash)?;
    let object_type = res_output.split(' ').collect::<Vec<&str>>()[0];
    let _size = res_output.split(' ').collect::<Vec<&str>>()[1];
    let size = _size.split('\0').collect::<Vec<&str>>()[0];

    if data_requested == "-t"{
        println!("{}", object_type);
    }
    if data_requested == "-s"{
        println!("{}", size);
    }
    if data_requested == "-p"{
        let raw_data = match res_output.split_once('\0') {
            Some((_object_type, raw_data)) => raw_data,
            None => {
                println!("Error: invalid object type");
                return Err(GitrError::FileReadError(object_hash.to_string()))
            }
        };

        match object_type {
            "blob" => print_blob_data(raw_data),
            "tree" => print_tree_data(raw_data),
            "commit" => print_commit_data(raw_data),
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


fn status_print_current_branch() -> Result<(), GitrError>{
    let head = file_manager::get_head()?;
    let current_branch = head.split('/').collect::<Vec<&str>>()[2];
    println!("On branch {}", current_branch);
    Ok(())
}

fn get_index_hashmap() -> Result<(HashMap<String, String>, bool), GitrError> {
    // index
    let mut index_hashmap = HashMap::new();
    //busco el index
    let mut hayindex = true;
    let index_data = match file_manager::read_index() {
        Ok(data) => data,
        Err(_) => {
            //let message = format!("\nNo commits yet\n\nnothing to commit (create/copy files and use \"git add\" to track)");
            //println!("{}", message);
            hayindex = false;
            String::new()
        }
    };
    // cargo el diccionario
    if hayindex {
        for index_entry in index_data.split('\n') {
            let attributes = index_entry.split(' ').collect::<Vec<&str>>();
            let path = attributes[3].to_string();
            let hash = attributes[1].to_string();
            index_hashmap.insert(path, hash);
        }
    }
    Ok((index_hashmap, hayindex))
}

fn get_current_commit_hashmap() -> Result<HashMap<String, String>, GitrError> {
      // current commit
      let mut tree_hashmap = HashMap::new();
      //busco el current commit
      let mut haycommitshechos = true;
      let current_commit = match file_manager::get_current_commit() {
          Ok(commit) => commit,
          Err(_) => {
              //let message = format!("\nNo commits yet\n\nnothing to commit (create/copy files and use \"git add\" to track)");
              //println!("{}", message);
              haycommitshechos = false;
              String::new()
          }
      };
      
      if haycommitshechos {
        
        let repo = file_manager::get_current_repo()?;
        let tree = file_manager::get_main_tree(current_commit)?;
        let tree_data = file_manager::read_object(&tree)?;
        let tree_entries = match tree_data.split_once('\0') {
            Some((_tree_type, tree_entries)) => tree_entries,
            None => "",
        };
          // cargo el diccionario
          
        for entry in tree_entries.split('\n') {
            let attributes = entry.split(' ').collect::<Vec<&str>>()[1];
            let _file_path= attributes.split('\0').collect::<Vec<&str>>()[0].to_string();
            let file_path = format!("{}/{}", repo, _file_path);
            let file_hash = attributes.split('\0').collect::<Vec<&str>>()[1].to_string();

            tree_hashmap.insert(file_path, file_hash);
        }

      }

      Ok(tree_hashmap)
}

pub fn get_working_dir_hashmap() -> Result<HashMap<String, String>, GitrError>{
    // working dir
    let mut working_dir_hashmap = HashMap::new();
    //busco el working dir
    let repo = file_manager::get_current_repo()?;
    let path = Path::new(repo.as_str());
    let files= command_utils::visit_dirs(path);
    //cargo el diccionario
    for file_path in files {
        let file_data = file_manager::read_file(file_path.clone())?;
        
        let blob = Blob::new(file_data.clone())?;
        let hash = blob.get_hash();
        working_dir_hashmap.insert(file_path, hash);
    }
    Ok(working_dir_hashmap)
}

pub fn status_print_to_be_comited(to_be_commited: &Vec<String>){
    if !to_be_commited.is_empty() {
        println!("Changes to be committed:");
        println!("  (use \"rm <file>...\" to unstage)");

        for file in to_be_commited.clone() {
            let file_name = match file.split_once ('/'){
                Some((_path, file)) => file.to_string(),
                None => file.to_string(),
            };
            println!("\t\x1b[92mmodified   {}\x1b[0m", file_name);
        }
    }
}

pub fn status_print_not_staged(not_staged: &Vec<String>) {
    if !not_staged.is_empty() {
        println!("Changes not staged for commit:");
        println!("  (use \"add <file>...\" to update what will be committed)");
        println!("  (use \"rm <file>...\" to discard changes in working directory)");

        for file in not_staged.clone() {
            let file_name = match file.split_once ('/'){
                Some((_path, file)) => file.to_string(),
                None => file,
            };
            println!("\t\x1b[31mmodified:   {}\x1b[0m", file_name);
        }
    }
}

pub fn status_print_untracked(untracked_files: &Vec<String>, hayindex: bool) {
    if !untracked_files.is_empty() {
        println!("Untracked files:");
        println!("  (use \"add <file>...\" to include in what will be committed)");

        for file in untracked_files.clone() {
            let file_name = match file.split_once ('/'){
                Some((_path, file)) => file.to_string(),
                None => file,
            };
            

            println!("\t\x1b[31m{}\x1b[0m", file_name);
        }

        if !hayindex {
            println!("nothing added to commit but untracked files present (use \"add\" to track)");
        }
    }
}

pub fn status(flags: Vec<String>) -> Result<(), GitrError>{
    status_print_current_branch()?;

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

    let index_path = &(repo.clone() + "/gitr/index");
    if Path::new(index_path).is_file() {
        
        let index_data = file_manager::read_index()?;

        let mut index_vector: Vec<&str> = Vec::new();

        if !index_data.is_empty() {
            index_vector = index_data.split('\n').collect::<Vec<&str>>();
        }

        let mut i: i32 = 0;
        while i != index_vector.len() as i32{
            
            let entry = index_vector[i as usize];
            let path_to_check = entry.split(' ').collect::<Vec<&str>>()[3];
            if !Path::new(path_to_check).exists(){
                index_vector.remove(i as usize);
                i -= 1;
            }
            i += 1;
        };
        
        
        file_manager::remove_file(format!("{}/gitr/index", repo))?;


        for entry in index_vector {
            let path = entry.split(' ').collect::<Vec<&str>>()[3];
            
            save_and_add_blob_to_index(path.to_string())?;
        }
        
    }

     
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

pub fn commit(flags: Vec<String>)-> Result<(), GitrError>{
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

pub fn checkout(flags: Vec<String>)->Result<(), GitrError> {
    if flags.len() != 1 {
        return Err(GitrError::InvalidArgumentError(flags.join(" "), "checkout <branch>".to_string()));
    }

    if !branch_exists(flags[0].clone()){
        println!("error: pathspec '{}' did not match any file(s) known to git.", flags[0]);
        return Ok(())
    }

    let current_commit = file_manager::get_commit(flags[0].clone())?;
    println!("curent commit = {}", current_commit);

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
    // let address = flags[0].clone();
    // let mut socket = clone_connect_to_server(address)?;
    // println!("clone():Servidor conectado.");
    // clone_send_git_upload_pack(&mut socket)?;
    // println!("clone():Envié upload-pack");
    // let ref_disc = clone_read_reference_discovery(&mut socket)?;
    // let references = ref_discovery::discover_references(ref_disc)?;
    // println!("clone():Referencias ={:?}=", references);
    // let want_message = ref_discovery::assemble_want_message(&references)?;
    // println!("clone():want {:?}", want_message);

    // socket.write(want_message.as_bytes())?;

    // let mut buffer = [0;1024];
    // socket.read(&mut buffer)?;
    // print!("clone(): recepeción de packfile:");
    // read_and_print_socket_read(&mut socket);

    // let objects = read_pack_file(&mut buffer)?;
    Ok(())
}

pub fn fetch(flags: Vec<String>) {
    println!("fetch");
}

pub fn merge(flags: Vec<String>) {
    println!("merge");
}

pub fn remote(flags: Vec<String>) {
    match file_manager::get_all_commits() {
        Ok(commits) => {
            
        },
        Err(_) => println!("Error: no commits found"),
    };
    println!("remote");
}

pub fn pull(flags: Vec<String>) {
    println!("pull");
}

pub fn push(flags: Vec<String>) {
    println!("push");
}

pub fn branch(flags: Vec<String>)->Result<(), GitrError>{
    if flags.is_empty() || (flags.len() == 1 && flags[0] == "-l") || (flags.len() == 1 && flags[0] == "--list"){
        match print_branches() {
            Ok(()) => (),
            Err(_) => return Err(GitrError::InvalidArgumentError(flags.join(" "), "TODO: escribir como se usa branch aca".into()))
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
            Err(_) => return Err(GitrError::InvalidArgumentError(flags.join(" "), "TODO: escribir como se usa branch aca".into()))
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