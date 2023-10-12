use std::{io::prelude::*, fs::{File, self}};
use flate2::Compression;
use flate2::write::ZlibEncoder;

use sha1::{Sha1, Digest};
/*
    NOTA: Puede que no todos los comandos requieran de flags,
    si ya esta hecha la funcion y no se uso, se puede borrar
    (y hay que modificar el llamado desde handler.rs tambien)
*/

fn sha1hashing(input: String) {
    let mut hasher = Sha1::new();
    hasher.update(input);
    let result = hasher.finalize();

}

fn flate2compress(input: String) -> Result<Vec<u8>, Box<dyn std::error::Error>>{
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(input.as_bytes())?;
    let compressed_bytes = encoder.finish()?;
    Ok(compressed_bytes)
}

/// Computes the object ID value for an object with the contents of the named file 
/// When <type> is not specified, it defaults to "blob".
pub fn hash_object(flags: Vec<String>) {
    let mut file_path = String::new();
    if flags.len() == 1 {
        file_path = flags[0].clone();
    }
    // USAR ?
    let mut file = match fs::read_to_string(file_path) {
        Ok(file) => file,
        Err(e) => panic!("Error: {}", e),
    };
    let mut compressed_file = match flate2compress(file) {
        Ok(compressed_bytes) => {
            compressed_bytes
        },
        Err(e) => panic!("Error: {}", e),
    };

    let mut hashed_file = match sha1hashing(compressed_file) {
        Ok(hashed_file) => hashed_file,
        Err(e) => panic!("Error: {}", e),
    };
    

    // create a Sha1 object
    let mut hasher = Sha1::new();

    // process input message
    hasher.update(b"hello world");
    // acquire hash digest in the form of GenericArray,
    // which in this case is equivalent to [u8; 20]
    let result = hasher.finalize();
    result.iter().for_each(|b| print!("{:02x}", b));
    println!("{:?}", result);
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
