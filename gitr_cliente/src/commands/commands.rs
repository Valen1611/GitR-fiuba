use std::{io::prelude::*, fs::{File, self}, error::Error};
use flate2::Compression;
use flate2::write::ZlibEncoder;
use crate::objects::blob::Blob;

use sha1::{Sha1, Digest};
/*
    NOTA: Puede que no todos los comandos requieran de flags,
    si ya esta hecha la funcion y no se uso, se puede borrar
    (y hay que modificar el llamado desde handler.rs tambien)
*/

fn sha1hashing(input: Vec<u8>) -> Vec<u8> {
    let mut hasher = Sha1::new();
    hasher.update(input);
    let result = hasher.finalize();
    result.to_vec()
}

fn flate2compress(input: String) -> Result<Vec<u8>, Box<dyn std::error::Error>>{
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
    
    let data = fs::read_to_string(file_path)?;
    
    let format_data = format!("blob {}\0{}", data.len(), data);


    /*
    100644 blob 6ff87c4664981e4397625791c8ea3bbb5f2279a3 file1
    100644 blob 3bb0e8592a41ae3185ee32266c860714980dbed7 file2
    tree <size-of-tree-in-bytes>\0
    <file-1-mode> <file-1-path>\0<file-1-blob-hash>
    <file-2-mode> <file-2-path>\0<file-2-blob-hash>
    ...
    <file-n-mode> <file-n-path>\0<file-n-blob-hash>
    

    commit <size-of-commit-data-in-bytes>'\0'
    <tree-SHA1-hash>
    <parent-1-commit-id>
    <parent-2-commit-id>
    ...
    <parent-N-commit-id>
    author ID email date
    committer ID email date

    user comment

     */

    let compressed_file = flate2compress(format_data)?;
    
    let hashed_file = sha1hashing(compressed_file);
    
    let hashed_file_str = hashed_file.iter().map(|b| format!("{:02x}", b)).collect::<String>();
    println!("{}", hashed_file_str);
    println!();

    if write {
        let blob = Blob::new(hashed_file_str);
        blob.save()?;
    }


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
