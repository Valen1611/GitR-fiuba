use std::{io::{Write, Read}, fs, path::Path, error::Error, collections::HashMap, net::TcpStream};

use flate2::Compression;
use flate2::write::ZlibEncoder;
use sha1::{Sha1, Digest};

use crate::{file_manager::{read_index, self}, objects::{blob::{TreeEntry, Blob}, tree::Tree, commit::Commit}};



pub fn sha1hashing(input: String) -> Vec<u8> {
    let mut hasher = Sha1::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    result.to_vec()
}

pub fn flate2compress(input: String) -> Result<Vec<u8>, Box<dyn std::error::Error>>{
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(input.as_bytes())?;
    let compressed_bytes = encoder.finish()?;
    Ok(compressed_bytes)
}

pub fn print_blob_data(raw_data: &str) {
    println!("{}", raw_data);
}

pub fn print_tree_data(raw_data: &str){
    let files = raw_data.split("\n").collect::<Vec<&str>>();
    
    for object in files {

        let file_atributes = object.split(" ").collect::<Vec<&str>>();
        let file_mode = file_atributes[0];
        let file_path_hash = file_atributes[1];
        
        let file_path = file_path_hash.split("\0").collect::<Vec<&str>>()[0];
        let file_hash = file_path_hash.split("\0").collect::<Vec<&str>>()[1];

        let mut file_type = "";  
        if file_mode == "100644"{
            file_type = "blob";
        } else{
            file_type = "tree";
        }
        //let file_path = file_atributes[1];

        //let file_hash = file_atributes[2];

        println!("{} {} {} {}", file_mode, file_type, file_hash, file_path);
        
    }
   
}

// commit <size-of-commit-data-in-bytes>'\0'
// <tree-SHA1-hash>
// <parent-1-commit-id>
// <parent-2-commit-id>
// ...
// <parent-N-commit-id>
// author ID email date
// committer ID email date


pub fn print_commit_data(raw_data: &str){
    println!("tree {}", raw_data);
}

pub fn visit_dirs(dir: &Path) -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    let mut subfiles = visit_dirs(&path);
                    files.append(&mut subfiles);
                } else if let Some(path_str) = path.to_str() {
                    files.push(path_str.to_string());
                    println!("{}", path.display());
                }
            }
        }
    }
    files
}

/*
100644 cde52ee64ce41d6cdd26720ea294ffb1c4c7835f 0 src/command_utils.rs
100644 3aa76051467c0484ced4aaf6e1c1645929b86bdd 0 src/commands/mod.rs
100644 e9a736e818411bd73d57a991423a00c128fbbd1c 0 src/commands/handler.rs
100644 5483d3ebb9a1d1c9d24f9f622f9513ab5e4636e7 0 src/commands/commands.rs
100644 ed4c92bffcd03151f42eae6440e5486ab1fd8227 0 src/objects/tree.rs
100644 681a8eab050aee5019f74412930c320d001b151d 0 src/objects/blob.rs
100644 91fe76f4819cabd9c66705f876411a5d1b92d979 0 src/objects/mod.rs
100644 67d10bb5a8777c10348f20c6e4a827eb1bdca43b 0 src/objects/commit.rs
100644 56c1f4951f789aa306f3f8db5fff8aabaa9c40ef 0 src/file_manager.rs
100644 8f5fedb7b69dd90f768c818ee085eca518e6520b 0 src/gitr_errors.rs
100644 782b63f6e510ac248a2a02dc1b002f315a3832f1 0 src/main.rs

{
src: [command_utils.rs, commands]
["src", "commands", "objects"] order
commands:[mods.rs, handler.rs, commands.rs]
objects:[tree.rs, blob.rs, mods.rs, commit.rs]
}
commit->tree
        |-src
            |-command_utils.rs
            |-main.rs
            |-gitr_errors.rs
            |-commands
                |-commands.rs
                |-handler.rs
                |-mod.rs
            |-objects
                |-tree.rs
                |-blob.rs
                |-mods.rs
                |-commit.rs
*/
pub fn create_trees (tree_map:HashMap<String, Vec<String>>, current_dir: String) -> Result<Tree, Box<dyn Error>> {
    let mut tree_entry: Vec<(String,TreeEntry)> = Vec::new();
    if let Some(objs) = tree_map.get(&current_dir) {
        for obj in objs {
                if tree_map.contains_key(obj) {
                    let new_tree = create_trees(tree_map.clone(), obj.to_string())?;
                    tree_entry.push((obj.clone(), TreeEntry::Tree(new_tree)));
            } else {
                let raw_data = fs::read_to_string(obj)?;
                let blob = Blob::new(raw_data)?;
                tree_entry.push((obj.clone(), TreeEntry::Blob(blob)));
            }
        }
    };

    let tree = Tree::new(tree_entry)?;
    tree.save()?;
    Ok(tree)
}

pub fn get_tree_entries(message:String) -> Result<(), Box<dyn Error>>{
    let mut tree_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut tree_order: Vec<String> = Vec::new(); // orden en el que insertamos carpetas
    let index_files = read_index()?;
    for file_info in index_files.split("\n") {
        let file_path = file_info.split(" ").collect::<Vec<&str>>()[3];
        let splitted_file_path = file_path.split("/").collect::<Vec<&str>>();
        for (i, dir) in (splitted_file_path.clone()).iter().enumerate() {
            if let Some(last_element) = splitted_file_path.last() {
                if dir == last_element {
                    if tree_map.contains_key(splitted_file_path[i-1]) {
                        match tree_map.get_mut(splitted_file_path[i-1]) {
                            Some(folder) => {
                                folder.push(file_path.to_string());
                            },
                            None => {
                                println!("No se encontro el folder");
                            }
                        }
                    }
                }else {
                        if !tree_map.contains_key(dir as &str) {
                            tree_map.insert(dir.to_string(), vec![]);
                            tree_order.push(dir.to_string());
                        }
                        if i == 0 {
                            continue;
                        }
                    if tree_map.contains_key(splitted_file_path[i-1]) {
                        match tree_map.get_mut(splitted_file_path[i-1]) {
                            Some(folder) => {
                                if !folder.contains(&dir.to_string()) {
                                    folder.push(dir.to_string());
                                }             
                            },
                            None => {
                                println!("No se encontro el folder");
                            }
                        }
                    }        
                } 
            }
        }
    }
    let tree_all = create_trees(tree_map, tree_order[0].clone())?;
    let final_tree = Tree::new(vec![(".".to_string(), TreeEntry::Tree(tree_all))])?;
    final_tree.save()?;
    let head = file_manager::get_head();
    let commit = Commit::new(final_tree.get_hash(), head.clone(), get_current_username(), get_current_username(), message)?;
    commit.save()?;
    if head == "None"{
        let _ = file_manager::write_file(String::from("gitr/refs/heads/master"), commit.get_hash());
    }else{
        let path = format!("gitr/{}", head);
        let _ = file_manager::write_file(path.clone(), commit.get_hash());
    }
    Ok(())
}


pub fn get_current_username() -> String{
    if let Some(username) = std::env::var_os("USER") {
        match username.to_str(){
            Some(username) => username.to_string(),
            None => String::from("User"),
        }
    } else{
        String::from("User")
    }
}

pub fn print_branches()-> Result<(), Box<dyn Error>>{
    let head = file_manager::get_head();
    let head_vec = head.split("/").collect::<Vec<&str>>();
    let head = head_vec[head_vec.len()-1];
    let branches = file_manager::get_branches()?;
        for branch in branches{
            if head == branch{
                let index_branch = format!("* {}", branch);
                println!("{}",index_branch);
                continue;
            }
            println!("{}", branch);
        }
    Ok(())
}

pub fn branch_exists(branch: String) -> bool{
    let branches = file_manager::get_branches();
    let branches = match branches{
        Ok(branches) => branches,
        Err(_) => return false,
    };
    for branch_name in branches{
        if branch_name == branch{
            return true;
        }
    }
    false
}

pub fn clone_connect_to_server(address: String)->Result<TcpStream,Box<dyn Error>>{
    let socket = TcpStream::connect(address)?;
    Ok(socket)
}

pub fn clone_send_git_upload_pack(socket: &mut TcpStream)->Result<usize, Box<dyn Error>>{
    match socket.write("0031git-upload-pack /mi-repo\0host=localhost:9418\0".as_bytes()){ //51 to hexa = 
        Ok(bytes) => Ok(bytes),
        Err(e) => Err(Box::new(e)),
    }
}

pub fn clone_read_reference_discovery(socket: &mut TcpStream)->Result<String, Box<dyn Error>>{
    let mut buffer = [0; 1024];
    let mut response = String::new();
    loop{
        let bytes_read = socket.read(&mut buffer)?;
        let received_message = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
        if bytes_read == 0 || received_message == "0000"{ 
            break;
        }
        response=String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
    }
    Ok(response)
}

#[cfg(test)]
// Esta suite solo corre bajo el Git Daemon que tiene Bruno, está hardcodeado el puerto y la dirección, además del repo remoto.
mod tests{
    use std::{net::TcpStream, io::{Write, Read}};

    use crate::git_transport::ref_discovery::discover_references;

    use super::*;

    #[test]
    fn test00_clone_connects_to_daemon_correctly(){
        assert!(clone_connect_to_server("localhost:9418".to_string()).is_ok());
    }

    #[test]
    fn test01_clone_send_git_upload_pack_to_daemon_correctly(){
        let mut socket = clone_connect_to_server("localhost:9418".to_string()).unwrap();
        assert_eq!(clone_send_git_upload_pack(&mut socket).unwrap(),49); //0x31 = 49
    }
    
    #[test]
    fn test02_clone_receive_daemon_reference_discovery_correctly(){
        let mut socket = clone_connect_to_server("localhost:9418".to_string()).unwrap();
        clone_send_git_upload_pack(&mut socket).unwrap();
        assert_eq!(clone_read_reference_discovery(&mut socket).unwrap(),"0103cf6335a864bda2ee027ea7083a72d10e32921b15 HEAD\0multi_ack thin-pack side-band side-band-64k ofs-delta shallow deepen-since deepen-not deepen-relative no-progress include-tag multi_ack_detailed symref=HEAD:refs/heads/main object-format=sha1 agent=git/2.34.1\n003dcf6335a864bda2ee027ea7083a72d10e32921b15 refs/heads/main\n");
    }

    #[test]	
    fn test03_clone_gets_reference_vector_correctly(){
        let mut socket = clone_connect_to_server("localhost:9418".to_string()).unwrap();
        clone_send_git_upload_pack(&mut socket).unwrap();
        let ref_disc = clone_read_reference_discovery(&mut socket).unwrap();
        assert_eq!(discover_references(ref_disc).unwrap(), 
        [("cf6335a864bda2ee027ea7083a72d10e32921b15".to_string(), "HEAD".to_string()), 
        ("cf6335a864bda2ee027ea7083a72d10e32921b15".to_string(), "refs/heads/main".to_string())]);
    }

    fn test04_clone_sends_wants_correctly(){
        
    }
}
