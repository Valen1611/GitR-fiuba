use std::{io::prelude::*, fs::{File, self}, error::Error};
use flate2::Compression;
use flate2::write::ZlibEncoder;

use sha1::{Sha1, Digest};
/*
    NOTA: Puede que no todos los comandos requieran de flags,
    si ya esta hecha la funcion y no se uso, se puede borrar
    (y hay que modificar el llamado desde handler.rs tambien)
*/

fn sha1hashing(input: String) -> Vec<u8> {
    let mut hasher = Sha1::new();
    hasher.update(input);
    let result = hasher.finalize();
    result.to_vec()
}

fn flate2compress(input: Vec<u8>) -> Result<Vec<u8>, Box<dyn std::error::Error>>{
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&input)?;
    let compressed_bytes = encoder.finish()?;
    Ok(compressed_bytes)
}

/// Computes the object ID value for an object with the contents of the named file 
/// When <type> is not specified, it defaults to "blob".
pub fn hash_object(flags: Vec<String>) -> Result<(), Box<dyn Error>>{
    let mut file_path = String::new();
    if flags.len() == 1 {
        file_path = flags[0].clone();
    }

    let file = fs::read_to_string(file_path)?;
    println!("file: \n{}", file);
    
    let hashed_file = sha1hashing(file);
    println!("hashed_file: ");
    hashed_file.iter().for_each(|b| print!("{:02x}", b));

    let compressed_file = flate2compress(hashed_file)?;
    println!("\ncompressed_file: ");
    compressed_file.iter().for_each(|b| print!("{:02x}", b));

    

    Ok(())
}

pub fn cat_file(flags: Vec<String>) {
    println!("cat_file");
}

pub fn init(flags: Vec<String>) {
    println!("init");
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
