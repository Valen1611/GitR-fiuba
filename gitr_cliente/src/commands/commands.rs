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

pub fn rm(flags: Vec<String>) {
    println!("rm");
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
src: [command_utils.rs, commands, objects, file_manager.rs, gitr_errors.rs, main.rs]
commands:[mod.rs, handler.rs, commands.rs]
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
pub fn commit(flags: Vec<String>) {
    get_tree_entries();
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

pub fn branch(flags: Vec<String>) {
    println!("branch");
}

pub fn ls_files(flags: Vec<String>) {
    if flags[0] == "--stage"{
        let res_output = file_manager::read_index().unwrap();
        println!("{}", res_output);
    }

}