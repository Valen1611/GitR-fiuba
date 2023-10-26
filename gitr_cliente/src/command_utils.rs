use std::{io::Write, fs, path::Path, error::Error, collections::HashMap};

use flate2::Compression;
use flate2::write::ZlibEncoder;
use sha1::{Sha1, Digest};

use crate::{file_manager::{read_index, self, get_current_commit}, objects::{blob::{TreeEntry, Blob}, tree::Tree, commit::Commit}, gitr_errors::GitrError};



pub fn sha1hashing(input: String) -> Vec<u8> {
    let mut hasher = Sha1::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    result.to_vec()
}

pub fn flate2compress(input: String) -> Result<Vec<u8>, GitrError>{
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    match encoder.write_all(input.as_bytes()) {
        Ok(_) => {},
        Err(e) => return Err(GitrError::CompressionError),
    };
    let compressed_bytes = match encoder.finish() {
        Ok(bytes) => bytes,
        Err(e) => return Err(GitrError::CompressionError),
    };
    Ok(compressed_bytes)
}

pub fn print_blob_data(raw_data: &str) {
    println!("{}", raw_data);
}



pub fn print_tree_data(raw_data: &str){
    let files = raw_data.split("\n").collect::<Vec<&str>>();
    println!("files: {:?} ", files);
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
    println!("{}", raw_data);
}

pub fn visit_dirs(dir: &Path) -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    if path.ends_with("gitr") {
                        continue;
                    }
                    let mut subfiles = visit_dirs(&path);
                    files.append(&mut subfiles);
                } else if let Some(path_str) = path.to_str() {
                    files.push(path_str.to_string());
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

/*

src -> commands -> commands.rs
    -> objects -> blob.rs
    -> hello.rs
*/


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
    let head = file_manager::get_head()?;
    let repo = file_manager::get_current_repo()?;
    if head == "None"{
        let dir = repo + "/gitr/refs/heads/master";
        let commit = Commit::new(final_tree.get_hash(), "None".to_string(), get_current_username(), get_current_username(), message)?;
        commit.save()?;
        let _ = file_manager::write_file(dir, commit.get_hash())?;
    }else{
        let dir = repo + "/gitr/" + &head;
        let current_commit = file_manager::get_current_commit()?;
        let commit = Commit::new(final_tree.get_hash(), current_commit, get_current_username(), get_current_username(), message)?;
        commit.save()?;
        let _ = file_manager::write_file(dir, commit.get_hash())?;
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
    let head = file_manager::get_head()?;
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
