use std::{io::prelude::*, fs::{File, self}, error::Error};
use flate2::Compression;
use flate2::write::ZlibEncoder;
use crate::{objects::blob::Blob, file_manager};
use crate::command_utils::*;



use sha1::{Sha1, Digest};
/*
    NOTA: Puede que no todos los comandos requieran de flags,
    si ya esta hecha la funcion y no se uso, se puede borrar
    (y hay que modificar el llamado desde handler.rs tambien)
*/


/// Computes the object ID value for an object with the contents of the named file 
/// When <type> is not specified, it defaults to "blob".
pub fn hash_object(flags: Vec<String>) -> Result<(), Box<dyn Error>>{
    // hash-object -w <file>
    // hash-object <file>

    let mut file_path = String::new();
    let mut write = false;

    if flags.len() == 1 {
        file_path = flags[0].clone();
    }

    if flags.len() == 2 {
        if flags[0] == "-w" {
            file_path = flags[1].clone();
            write = true;
        }
    }
    let raw_data = fs::read_to_string(file_path)?;
    
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


///tree <size-of-tree-in-bytes>\0
// <file-1-mode> <file-1-path>\0<file-1-blob-hash>
// <file-2-mode> <file-2-path>\0<file-2-blob-hash>
// ...
// <file-n-mode> <file-n-path>\0<file-n-blob-hash>



// commit <size-of-commit-data-in-bytes>'\0'
// <tree-SHA1-hash>
// <parent-1-commit-id>
// <parent-2-commit-id>
// ...
// <parent-N-commit-id>
// author ID email date
// committer ID email date

// user comment
pub fn cat_file(flags: Vec<String>) -> Result<(),Box<dyn Error>> {
    if flags.len() != 2 {
        println!("Error: invalid number of arguments");
        return Ok(())
    } 
    let res_output = file_manager::read_object(&flags[1])?;
    let object_type = res_output.split(" ").collect::<Vec<&str>>()[0];
    let _size = res_output.split(" ").collect::<Vec<&str>>()[1];
    let size = _size.split("\0").collect::<Vec<&str>>()[0];

    
    if flags[0] == "-t"{
        println!("{}", object_type);
    }
    if flags[0] == "-s"{
        println!("{}", size);
    }
    if flags[0] == "-p"{
        
        let raw_data_index = match res_output.find("\0") {
            Some(index) => index,
            None => {
                println!("Error: invalid object type");
                return Ok(())
            }
        };

        let raw_data = &res_output[(raw_data_index + 1)..];
        println!("object type: {}", object_type);
        match object_type {
            "blob" => print_blob_data(raw_data),
            "tree" => print_tree_data(raw_data),
            "commit" => println!("{}", res_output.split("\0").collect::<Vec<&str>>()[1]),
            _ => println!("Error: invalid object type"),
        }
    }
    


    let info_data = res_output.split("\0").collect::<Vec<&str>>();

    let type_size = info_data[0].split(" ").collect::<Vec<&str>>();    
    let object_type = type_size[0];
    let size = type_size[1];
    let raw_data = info_data[1];

    
    Ok(())

}

pub fn init(flags: Vec<String>) -> Result<(), Box<dyn Error>> {
    file_manager::init_repository(&flags[0])?;
    println!("Initialized empty Gitr repository");
    Ok(())
}

pub fn status(flags: Vec<String>) {
    println!("status");
}

pub fn add(flags: Vec<String>)-> Result<(), Box<dyn Error>> {
    if flags.len() != 1 {
        println!("Error: invalid number of arguments");
        return Ok(())
    }
    // check if flags[0] is an existing file
    let file_path = &flags[0];
    if file_path == "."{
        let files = visit_dirs(&std::path::Path::new("src"));
        for file in files{
            let raw_data = fs::read_to_string(file.clone())?;
            let blob = Blob::new(raw_data)?;
            blob.save()?;
            let hash = blob.get_hash();
            file_manager::add_to_index(&file, &hash)?;
        }
    }else{
        let raw_data = fs::read_to_string(file_path)?;
        let blob = Blob::new(raw_data)?;
        blob.save()?;
        let hash = blob.get_hash();
        file_manager::add_to_index(file_path, &hash)?;
    }
    Ok(())
    
}

pub fn rm(flags: Vec<String>)-> Result<(), Box<dyn Error>> {
    let mut removed:bool = false;
    if flags.len() != 1 {
        println!("Error: invalid number of arguments");
        return Ok(())
    }
    let mut index = file_manager::read_index()?;
    index = index + "\n";
    for line in index.lines(){
        let attributes = line.split(" ").collect::<Vec<&str>>();
        if attributes[3] == flags[0]{
            let complete_line = format!("{}\n", line);
            index = index.replace(&complete_line, "");
            let res = index.trim_end().to_string();
            removed = true;
            let compressed_index = flate2compress(res)?;
            let _ = file_manager::write_compressed_data("gitr/index", compressed_index.as_slice());
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


pub fn commit(flags: Vec<String>) {
    if flags.len() != 2 || flags[0] != "-m"{
        println!("Error: invalid number of arguments");
        return
    }
    let _ = get_tree_entries(flags[1].clone());
}

pub fn checkout(flags: Vec<String>) {
    println!("checkout");
}

pub fn log(flags: Vec<String>) {
    println!("log");
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
    if flags.len() == 0 || (flags.len() == 1 && flags[0] == "-l") || (flags.len() == 1 && flags[0] == "--list"){
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

        
        let old_path = format!("gitr/refs/heads/{}", flags[1]);
        let new_path = format!("gitr/refs/heads/{}", flags[2]);
        file_manager::move_branch(old_path.clone(), new_path.clone())?;
        let _ = file_manager::update_head(&new_path);
        return Ok(())

    }
    if flags.len() == 1 && flags[0] != "-l" && flags[0] != "--list"{
        if branch_exists(flags[0].clone()){
            println!("fatal: A branch named '{}' already exists.", flags[0]);
            return Ok(())
        }
        let current_commit = file_manager::get_current_commit()?;
        let _ = file_manager::write_file(format!("gitr/refs/heads/{}", flags[0]), current_commit);
    }
    Ok(())
}

pub fn ls_files(flags: Vec<String>) {
    if flags[0] == "--stage"{
        let res_output = file_manager::read_index().unwrap();
        println!("{}", res_output);
    }
}