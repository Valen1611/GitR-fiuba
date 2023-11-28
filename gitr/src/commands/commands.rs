


use std::path::Path;
use crate::file_manager::{get_branches, get_current_repo};
use crate::command_utils::{*, self};
use std::net::TcpStream;
use std::io::prelude::*;
use crate::file_manager::{commit_log, update_working_directory, get_current_commit};
use crate::objects::git_object::GitObject::*;
use crate::{objects::blob::Blob, objects::commit::Commit, file_manager, gitr_errors::GitrError, git_transport::pack_file::PackFile};
use crate::git_transport::pack_file::{create_packfile};
use crate::{git_transport};
use crate::git_transport::ref_discovery;

/***************************
 *************************** 
 *      COMMANDS
 **************************
 **************************/


//Create an empty Gitr repository 
pub fn init(flags: Vec<String>,cliente: String) -> Result<(), GitrError> {
    // init <name-of-new-repo>
    if flags.is_empty() || flags.len() > 1  {
        return Err(GitrError::InvalidArgumentError(flags.join(" "), "init <new_repo_name>".to_string()));
    }
    file_manager::init_repository(&(cliente.clone() +"/"+ &flags[0]))?;
    file_manager::update_current_repo(&flags[0],cliente)?;
    println!("Initialized empty Gitr repository");
    Ok(())
}

/// Computes the object ID value for an object with the contents of the named file 
pub fn hash_object(flags: Vec<String>,cliente: String) -> Result<(), GitrError>{
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
    file_path = file_manager::get_current_repo(cliente.clone())?.to_string() + "/" + &file_path;
    let raw_data = file_manager::read_file(file_path)?;  
    let blob = Blob::new(raw_data)?;
    println!("{}", blob.get_hash());
    if write {
        blob.save(cliente)?;
    }
    Ok(())
}


//Output the contents or other properties such as size, type or delta information of an object 
pub fn cat_file(flags: Vec<String>,cliente: String) -> Result<(),GitrError> {
    //cat-file -p <object-hash>
    //cat-file -t <object-hash>
    //cat-file -s <object-hash>
    if flags.len() != 2 {
        let flags_str = flags.join(" ");
        return Err(GitrError::InvalidArgumentError(flags_str,"cat-file <[-t/-s/-p]> <object hash>".to_string()));
    }
    let (object_hash, res_output, size, object_type) = get_object_properties(flags.clone(),cliente)?;
    print_cat_file_command(&flags[0].clone(), &object_hash, &object_type, res_output.clone(), &size)?;
    Ok(())
}


//Add file contents to the index
pub fn add(flags: Vec<String>, cliente: String)-> Result<(), GitrError> {
    // add <file-name>
    // add .
    if flags.len() != 1 {
        return Err(GitrError::InvalidArgumentError(flags.join(" "), "add <[file/.]>".to_string()))
    }
    update_index_before_add(cliente.clone())?;
    add_files_command(flags[0].clone(),cliente)?;
    Ok(())
    
}
//Remove files from the index
pub fn rm(flags: Vec<String>,cliente: String)-> Result<(), GitrError> {
    //rm <file>
    if flags.len() != 1 {
        return Err(GitrError::InvalidArgumentError(flags.join(" "), "rm <file>".to_string()))
    }
    let removed = rm_from_index(&flags[0],cliente)?;
    if removed{
        println!("rm '{}'", flags[0]);
    }else{
        println!("Error: file not found");
    }
    Ok(())
} 

//Record changes to the repository
pub fn commit(flags: Vec<String>, second_parent: String, cliente: String)-> Result<(), GitrError>{
    //commit -m <message-of-commit>
    if flags[0] != "-m" || flags.len() < 2 {
        return Err(GitrError::InvalidArgumentError(flags.join(" "), "commit -m <commit_message>".to_string()))
    }
    let index_path = file_manager::get_current_repo(cliente.clone())?.to_string() + "/gitr/index";
    if !Path::new(&index_path).exists() {
        return status(flags,cliente.clone());
    }
    let (not_staged, _, _) = get_untracked_notstaged_files(cliente.clone())?;
    let to_be_commited = get_tobe_commited_files(&not_staged,cliente.clone())?;
    println!("to be commited: {to_be_commited:?}");
    if to_be_commited.is_empty() {
        println!("nothing to commit, working tree clean");
        return Ok(())
    }
    if flags[1].starts_with('\"'){
        let message = &flags[1..];
        let message = message.join(" ");
        if !message.chars().any(|c| c!= ' ' && c != '\"'){
            return Err(GitrError::InvalidArgumentError(flags.join(" "), "commit -m \"commit_message\"".to_string()))
        }
        get_tree_entries(message.to_string(),second_parent, cliente.clone())?;
        print_commit_confirmation(message,cliente.clone())?;
        Ok(())
    } else {
        Err(GitrError::InvalidArgumentError(flags.join(" "), "commit -m \"commit_message\"".to_string()))
    }
}

// Switch branches or restore working tree files
pub fn checkout(flags: Vec<String>,cliente: String)->Result<(), GitrError> {
    if flags.is_empty() || flags.len() > 2 || (flags.len() == 2 && flags[0] != "-b"){
        return Err(GitrError::InvalidArgumentError(flags.join(" "), "checkout <branch>".to_string()));
    }
    commit_existing(cliente.clone())?;
    let branch_to_checkout = get_branch_to_checkout(flags.clone(),cliente.clone())?;
    let current_commit = file_manager::get_commit(branch_to_checkout.clone(),cliente.clone())?;
    file_manager::update_working_directory(current_commit,cliente.clone())?;
    let path_head = format!("refs/heads/{}", branch_to_checkout);
    file_manager::update_head(&path_head,cliente.clone())?;
    
    Ok(())
}

//Show commit logs
pub fn log(flags: Vec<String>,cliente: String)->Result<(), GitrError> {
    // log 
    commit_existing(cliente.clone())?;
    if flags.is_empty() {
       let log_res = commit_log("-1".to_string(),cliente.clone())?;
       print!("{}", log_res);
    }
    if flags.len() == 2 && flags[0] == "-n" && flags[1].parse::<usize>().is_ok(){
        let log_res = commit_log(flags[1].to_string(),cliente.clone())?;
        print!("{}", log_res);
    }
    Ok(())
}

// List, create, or delete branches
pub fn branch(flags: Vec<String>,cliente: String)->Result<(), GitrError>{
    //branch -m <origin_branch> <destination-branch>
    //branch
    //branch -d <branch-to-delete>
    //branch -l
    //branch <new-branch-name>
    if flags.is_empty() || (flags.len() == 1 && flags[0] == "-l") || (flags.len() == 1 && flags[0] == "--list"){
        print_branches(cliente.clone())?;
        return Ok(())
    }
    commit_existing(cliente.clone())?;
    if flags.len() == 2 && flags[0] == "-d"{
        branch_delete_flag(flags[1].clone(),cliente.clone())?;
    }
    if flags.len() == 3 && flags[0] == "-m"{
        branch_move_flag(flags[1].clone(), flags[2].clone(),cliente.clone())?;
    }
    if flags.len() == 1 && flags[0] != "-l" && flags[0] != "--list"{
        branch_newbranch_flag(flags[0].clone(),cliente.clone())?;
    }
    Ok(())
}

pub fn ls_files(flags: Vec<String>,cliente: String) -> Result<(), GitrError>{
    //ls-files --stage
    if flags.len() == 0 || flags[0] == "--cached" || flags[0] == "-c" {
        let ls_files_res = get_ls_files_cached(cliente.clone())?;
        print!("{}", ls_files_res);
        return Ok(())
    }
    if flags[0] == "--stage"{
        let res_output = file_manager::read_index(cliente.clone())?;
        println!("{}", res_output);
        return Ok(())
    }
    if flags[0] == "--deleted"{
        let res_output = get_ls_files_deleted_modified(true,cliente.clone())?;
        print!("{}", res_output);
        return Ok(())
    }
    if flags[0] == "--modified"{
        let res_output = get_ls_files_deleted_modified(false,cliente.clone())?;
        print!("{}", res_output);
        return Ok(())
    }
    Err(GitrError::InvalidArgumentError(flags.join(" "),"ls-files --stage".to_string() ))
}

pub fn clone(flags: Vec<String>,cliente: String)->Result<(),GitrError>{
    init(vec![flags[1].clone()],cliente.clone())?;
    remote(vec![flags[0].clone()],cliente.clone())?;
    pullear(vec![],true,cliente)?;
    Ok(())
}

// Show the working tree status
pub fn status(_flags: Vec<String>,cliente: String) -> Result<(), GitrError>{
    command_utils::status_print_current_branch(cliente.clone())?;

    let (not_staged, untracked_files, hayindex) = get_untracked_notstaged_files(cliente.clone())?;
    let to_be_commited = get_tobe_commited_files(&not_staged,cliente.clone())?;
    print!("{}", get_status_files_to_be_comited(&to_be_commited)?);

    print!("{}", get_status_files_not_staged(&not_staged,cliente.clone())?);
    print!("{}",get_status_files_untracked(&untracked_files, hayindex));
    if to_be_commited.is_empty() && not_staged.is_empty() && untracked_files.is_empty() {
        println!("nothing to commit, working tree clean");
    }
    Ok(())
}


pub fn fetch(flags: Vec<String>,cliente: String) -> Result<(), GitrError>{
    pullear(flags, false,cliente)
}

pub fn merge(_flags: Vec<String>,cliente: String) -> Result<(), GitrError>{
    if _flags.is_empty(){
        return Err(GitrError::InvalidArgumentError(_flags.join(" "), "merge <branch-name>".to_string()))
    }

    let branch_name = _flags[0].clone();
    let origin_name = file_manager::get_head(cliente.clone())?.split('/').collect::<Vec<&str>>()[2].to_string();

    let branch_commits = command_utils::branch_commits_list(branch_name.clone(),cliente.clone())?;
    let origin_commits = command_utils::branch_commits_list(origin_name,cliente.clone())?;
    for commit in branch_commits.clone() {
        if origin_commits.contains(&commit) {
            if commit == origin_commits[0] {
                println!("Updating {}..{}" ,&origin_commits[0][..7], &branch_commits[0][..7]);
                println!("Fast-forward");
                command_utils::fast_forward_merge(branch_name,cliente.clone())?;
                break;
            }
            command_utils::three_way_merge(commit, origin_commits[0].clone(), branch_commits[0].clone(), branch_name, cliente.clone())?;
            break;
        }
    }
    Ok(())
}

pub fn remote(flags: Vec<String>,cliente: String) -> Result<(), GitrError> {
    if flags.is_empty() {
        let remote = file_manager::read_file(get_current_repo(cliente.clone())? + "/gitr/remote")?;
        println!("remote: {}",remote);
    } else {
        file_manager::write_file(get_current_repo(cliente.clone())? + "/gitr/remote", flags[0].clone())?;
    }
    Ok(())
}

fn pullear(flags: Vec<String>, actualizar_work_dir: bool,cliente: String) -> Result<(), GitrError> {
    if !flags.is_empty(){
        return Err(GitrError::InvalidArgumentError(flags.join(" "), "pull <no-args>".to_string()));
    }

    // ########## HANDSHAKE ##########
    let _repo = file_manager::get_current_repo(cliente.clone())?;
    let remote = file_manager::get_remote(cliente.clone())?;
    let msj = format!("git-upload-pack /{}\0host={}\0","mi-repo", remote);
    let msj = format!("{:04x}{}", msj.len() + 4, msj);
    let mut stream = match TcpStream::connect("localhost:9418") {
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
                if n == 0 {
                    return Ok(());
                }
                let bytes = &buffer[..n];
                let s = String::from_utf8_lossy(bytes);
                ref_disc.push_str(&s);
                if s.ends_with("0000") {
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
    let want_message = ref_discovery::assemble_want_message(&hash_n_references,file_manager::get_heads_ids(cliente.clone())?,cliente.clone())?;
    file_manager::update_client_refs(hash_n_references.clone(), file_manager::get_current_repo(cliente.clone())?)?;
    match stream.write(want_message.as_bytes()) {
        Ok(_) => (),
        Err(e) => {
            println!("Error: {}", e);
            return Ok(())
        }
    };
    if want_message == "0000" {
        println!("cliente al día");
        return Ok(())
    }
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
    // ########## PACKFILE ##########
    let mut buffer = Vec::new();
    let n = match stream.read_to_end(&mut buffer) { // Leo el packfile
        Err(e) => {
            println!("Error: {}", e);
            return Ok(())
        },
        Ok(n) => n
    };
    let pack_file_struct = PackFile::new_from_server_packfile(&mut buffer[..n])?;
    for object in pack_file_struct.objects.iter(){
        match object{
            Blob(blob) => blob.save(cliente.clone())?,
            Commit(commit) => commit.save(cliente.clone())?,
            Tree(tree) => tree.save(cliente.clone())?,
        }
    }
    if actualizar_work_dir {
        update_working_directory(get_current_commit(cliente.clone())?,cliente.clone())?;
    }
    println!("pull successfull");
    Ok(())
}

pub fn pull(flags: Vec<String>,cliente: String) -> Result<(), GitrError> {
   pullear(flags, true,cliente)
}

pub fn push(flags: Vec<String>,cliente: String) -> Result<(),GitrError> {
    if !flags.is_empty(){
        return Err(GitrError::InvalidArgumentError(flags.join(" "), "push <no-args>".to_string()));
    }
    // ########## HANDSHAKE ##########
    let repo = file_manager::get_current_repo(cliente.clone())?;
    let remote = file_manager::get_remote(cliente.clone())?;
    let msj = format!("git-receive-pack /{}\0host={}\0","mi-repo", remote);
    let msj = format!("{:04x}{}", msj.len() + 4, msj);
    let mut stream = match TcpStream::connect("localhost:9418") {
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
                if n == 0 {
                    return Ok(());
                }
                let bytes = &buffer[..n];
                let s = String::from_utf8_lossy(bytes);
                ref_disc.push_str(&s);
                if s.ends_with("0000") {
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
    // ########## REFERENCE UPDATE REQUEST ##########
    let ids_propios = file_manager::get_heads_ids(cliente.clone())?; // esta sacando de gitr/refs/heads
    let refs_propios = get_branches(cliente.clone())?; // tambien de gitr/refs/heads
    let (ref_upd,pkt_needed,pkt_ids) = ref_discovery::reference_update_request(hash_n_references.clone(),ids_propios,refs_propios)?;
    if let Err(e) = stream.write(ref_upd.as_bytes()) {
        println!("Error: {}", e);
        return Ok(())
    };
    if ref_upd == "0000" {
        println!("client up to date");
        return Ok(())
    }
    if pkt_needed {
        let all_pkt_commits = Commit::get_parents(pkt_ids.clone(),hash_n_references.iter().map(|t|t.0.clone()).collect(),repo + "/gitr")?;
        let repo = file_manager::get_current_repo(cliente.clone())? + "/gitr";
        let ids = Commit::get_objects_from_commits(all_pkt_commits,vec![],repo.clone())?;
        let mut contents: Vec<Vec<u8>> = Vec::new();
        for id in ids {
            contents.push(file_manager::get_object_bytes(id, repo.clone())?)
        }
        let cont: Vec<(String, String, Vec<u8>)> = git_transport::pack_file::prepare_contents(contents.clone());
        let pk = create_packfile(cont.clone())?;
        if let Err(e) = stream.write(&pk) { // Mando el Packfile
            println!("Error: {}", e);
            return Ok(())
        };
    }
    Ok(())
}

pub fn show_ref(flags: Vec<String>,cliente: String) -> Result<(),GitrError> {
    if flags.len() > 1 || (flags.len() == 1 && flags[0] != "--head" ){
        return Err(GitrError::InvalidArgumentError(flags.join(" "), "show-ref [--head]".to_string()));
    }
    let refs = match ref_discovery::ref_discovery(&(get_current_repo(cliente.clone())? + "/gitr")) {
        Ok(refs) => refs.0,
        Err(e) => {
            println!("Error: {}", e);
            return Ok(())
        }
    };
    let refs = refs.strip_suffix("0000").unwrap_or(&refs);
    if flags.contains(&"--head".to_string()) {
        println!("{}",refs);
        return Ok(());
    }
    let mut first = true;
    for line in refs.lines() {
        if first {
            first = false;
            continue;
        }
        println!("{}", line);
    }
    Ok(())
}

pub fn list_repos(cliente: String) {
    println!("{:?}", file_manager::get_repos(cliente.clone()));
}

pub fn go_to_repo(flags: Vec<String>,cliente: String) -> Result<(), GitrError>{
    if flags.len() != 1 {
        return Err(GitrError::InvalidArgumentError(flags.join(" "), "go-to-repo <repo>".to_string()));   
    }
    let new_repo = flags[0].clone();
    let existing_repos = file_manager::get_repos(cliente.clone());
    if existing_repos.contains(&new_repo) {
        file_manager::update_current_repo(&new_repo,cliente)?;
    }
    else {
        println!("Error: repository '{}' does not exist", new_repo);
    }
    Ok(())
}

pub fn print_current_repo(cliente: String) -> Result<(), GitrError> {
    let repo = file_manager::get_current_repo(cliente.clone())?;
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
        flags.push("repo_clonado".to_string());
        assert!(clone(flags,"test".to_string()).is_ok());
    }

}
