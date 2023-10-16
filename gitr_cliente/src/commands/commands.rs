use std::{io::prelude::*, fs::{File, self}, error::Error};
use flate2::Compression;
use flate2::write::ZlibEncoder;
use crate::{objects::blob::Blob, file_manager};

use sha1::{Sha1, Digest};
/*
    NOTA: Puede que no todos los comandos requieran de flags,
    si ya esta hecha la funcion y no se uso, se puede borrar
    (y hay que modificar el llamado desde handler.rs tambien)
*/

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

pub fn cat_file(flags: Vec<String>) {
    if flags.len() != 2  || flags[0] != "-p" || flags[0] != "-t" {
        println!("Error: invalid number of arguments");
        return;
    }
    

}

pub fn init(flags: Vec<String>) -> Result<(), Box<dyn Error>> {
    file_manager::init_repository(&flags[0])?;
    println!("Initialized empty Gitr repository");
    Ok(())
}

pub fn status(flags: Vec<String>) {
    println!("status");
}

pub fn add(flags: Vec<String>) {
    println!("add");
    println!("flags: {:?}", flags);
}

pub fn rm(flags: Vec<String>) {
    println!("rm");
} 

pub fn commit(flags: Vec<String>) {
    println!("commit");
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
