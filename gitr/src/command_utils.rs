
use std::{io::{Write, Read}, fs::{self}, path::Path, collections::HashMap, net::TcpStream};

use flate2::Compression;
use flate2::write::ZlibEncoder;
use sha1::{Sha1, Digest};


use crate::file_manager::{read_index, self, get_head, get_current_commit};
use crate::{objects::{blob::{TreeEntry, Blob}, tree::Tree, commit::Commit}, gitr_errors::GitrError};

pub fn flate2compress2(input: Vec<u8>) -> Result<Vec<u8>, GitrError>{
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    
    match encoder.write_all(&input) {
        Ok(_) => {},
        Err(_) => return Err(GitrError::CompressionError),
    };
    
    let compressed_bytes = match encoder.finish() {
        Ok(bytes) => bytes,
        Err(_) => return Err(GitrError::CompressionError),
    };
    Ok(compressed_bytes)
}

pub fn sha1hashing2(input: Vec<u8>) -> Vec<u8> {
    let mut hasher = Sha1::new();
    hasher.update(&input);
    let result = hasher.finalize();
    result.to_vec()
}

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
        Err(_) => return Err(GitrError::CompressionError),
    };
    let compressed_bytes = match encoder.finish() {
        Ok(bytes) => bytes,
        Err(_) => return Err(GitrError::CompressionError),
    };
    Ok(compressed_bytes)
}

pub fn print_blob_data(raw_data: &str) {
    println!("{}", raw_data);
}

pub fn print_tree_data(raw_data: &str) {
    let files = raw_data.split('\n').collect::<Vec<&str>>();

    for object in files {
        let file_atributes = object.split(' ').collect::<Vec<&str>>();
        let file_mode = file_atributes[0];
        let file_path_hash = file_atributes[1];
        
        let file_path = file_path_hash.split('\0').collect::<Vec<&str>>()[0];
        let file_hash = file_path_hash.split('\0').collect::<Vec<&str>>()[1];

        let file_type = if file_mode == "100644"{
            "blob"
        } else{
            "tree"
        };

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
        for entry in entries.flatten() {
          
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

pub fn create_trees(tree_map:HashMap<String, Vec<String>>, current_dir: String) -> Result<Tree, GitrError> {
    let mut tree_entry: Vec<(String,TreeEntry)> = Vec::new();
    if let Some(objs) = tree_map.get(&current_dir) {
        for obj in objs {
                if tree_map.contains_key(obj) {
                    let new_tree = create_trees(tree_map.clone(), obj.to_string())?;
                    tree_entry.push((obj.clone(), TreeEntry::Tree(new_tree)));
            } else {
                //let raw_data = fs::read_to_string(obj)?;
                let raw_data = file_manager::read_file(obj.clone())?;
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
                -> handler.rs
                
    -> objects -> blob.rs
    -> hello.rs
*/


pub fn get_tree_entries(message:String) -> Result<(), GitrError>{
    let mut tree_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut tree_order: Vec<String> = Vec::new(); // orden en el que insertamos carpetas
    let index_files = read_index()?;
    for file_info in index_files.split('\n') {
        let file_path = file_info.split(' ').collect::<Vec<&str>>()[3];
        let splitted_file_path = file_path.split('\\').collect::<Vec<&str>>();
        println!("{}",file_path);
        for (i, dir) in (splitted_file_path.clone()).iter().enumerate() {
            if let Some(last_element) = splitted_file_path.last() { //es el ultimo?
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

    

    let final_tree = create_trees(tree_map, tree_order[0].clone())?;

   // println!("tree_all: {:?}", tree_all);
    
    //let final_tree = Tree::new(vec![(".".to_string(), TreeEntry::Tree(tree_all))])?;
    
    //println!("final_tree: {:?}", final_tree);

    final_tree.save()?;
    let head = file_manager::get_head()?;
    let repo = file_manager::get_current_repo()?;
 

    let path_completo = repo.clone()+"/gitr/"+head.as_str();
    
    if fs::metadata(path_completo.clone()).is_err(){

        let dir = repo + "/gitr/refs/heads/master";
        file_manager::write_file(path_completo, final_tree.get_hash())?;
        if !Path::new(&dir).exists(){
            let current_commit = file_manager::get_current_commit()?;
            file_manager::write_file(dir.clone(), current_commit)?;
        }
        
        let commit = Commit::new(final_tree.get_hash(), "None".to_string(), get_current_username(), get_current_username(), message)?;
        commit.save()?;
        file_manager::write_file(dir, commit.get_hash())?;
    }else{

        let dir = repo + "/gitr/" + &head;
        let current_commit = file_manager::get_current_commit()?;
        
        let commit = Commit::new(final_tree.get_hash(), current_commit, get_current_username(), get_current_username(), message)?;
        commit.save()?;
        file_manager::write_file(dir, commit.get_hash())?;
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

pub fn print_branches()-> Result<(), GitrError>{
    let head = file_manager::get_head()?;
    let head_vec = head.split('/').collect::<Vec<&str>>();
    let head = head_vec[head_vec.len()-1];
    let branches = file_manager::get_branches()?;
        for branch in branches{
            if head == branch{
                let index_branch = format!("* \x1b[92m{}\x1b[0m", branch);
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

pub fn print_commit_confirmation(message:String)->Result<(), GitrError>{
    let branch = get_head()?
            .split('/')
            .collect::<Vec<&str>>()[2]
            .to_string();
        let hash_recortado = &get_current_commit()?[0..7];

        println!("[{} {}] {}", branch, hash_recortado, message);
        Ok(())
}

pub fn clone_connect_to_server(address: String)->Result<TcpStream,GitrError>{
    let socket = match TcpStream::connect(address) {
        Ok(socket) => socket,
        Err(_) => return Err(GitrError::InvalidArgumentError("address".to_string(), "localhost:9418".to_string())),
    };
    Ok(socket)
}

pub fn clone_send_git_upload_pack(socket: &mut TcpStream)->Result<usize, GitrError>{
    // let msj = format!("git-upload-pack /{}\0host={}\0",file_manager::get_current_repo()?, file_manager::get_remote()?);
    // let msj = format!("{:04x}{}", msj.len() + 4, msj);    
    // match socket.write(msj.as_bytes()){ //51 to hexa = 
    //     Ok(bytes) => Ok(bytes),
    //     Err(e) => Err(GitrError::ConnectionError),
    match socket.write("0031git-upload-pack /mi-repo\0host=localhost:9418\0".as_bytes()){ //51 to hexa = 
        Ok(bytes) => Ok(bytes),
        Err(e) => Err(GitrError::SocketError("clone_send_git_upload_pack()".to_string(), e.to_string())),
    }
}

pub fn clone_read_reference_discovery(socket: &mut TcpStream)->Result<String, GitrError>{
    let mut buffer = [0; 1024];
    let mut response = String::new();
    loop{
        let bytes_read = match socket.read(&mut buffer){
            Ok(bytes) => bytes,
            Err(e) => return Err(GitrError::SocketError("clone_read_reference_discovery()".to_string(), e.to_string())),
        };
        let received_message = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
        if bytes_read == 0 || received_message == "0000"{ 
            break;
        }
        response.push_str(&received_message);
    }
    Ok(response)
}

pub fn write_socket(socket: &mut TcpStream, message: &[u8])->Result<(),GitrError>{
    match socket.write(message){
        Ok(_) => Ok(()),
        Err(e) => Err(GitrError::SocketError("write_socket()".to_string(), e.to_string())),
    }
}

pub fn read_socket(socket: &mut TcpStream, buffer: &mut [u8])->Result<(),GitrError>{
    let bytes_read = match socket.read(buffer){
        Ok(bytes) => bytes,
        Err(e) => return Err(GitrError::SocketError("read_socket()".to_string(), e.to_string())),
    };
    let received_data = String::from_utf8_lossy(&buffer[..bytes_read]);
    println!("String recibido de tamaño {}: {:?}", bytes_read, received_data);
    Ok(())
}

#[cfg(test)]
// Esta suite solo corre bajo el Git Daemon que tiene Bruno, está hardcodeado el puerto y la dirección, además del repo remoto.
mod tests{
    use crate::git_transport::ref_discovery::{self, assemble_want_message};

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
        assert_eq!(ref_discovery::discover_references(ref_disc).unwrap(), 
        [("cf6335a864bda2ee027ea7083a72d10e32921b15".to_string(), "HEAD".to_string()), 
        ("cf6335a864bda2ee027ea7083a72d10e32921b15".to_string(), "refs/heads/main".to_string())]);
    }
    
    #[test]
    fn test04_clone_sends_wants_correctly(){
        let mut socket = clone_connect_to_server("localhost:9418".to_string()).unwrap();
        clone_send_git_upload_pack(&mut socket).unwrap();
        let ref_disc = clone_read_reference_discovery(&mut socket).unwrap();
        let references = ref_discovery::discover_references(ref_disc).unwrap();
        socket.write(assemble_want_message(&references,vec![]).unwrap().as_bytes()).unwrap();
    }
}
