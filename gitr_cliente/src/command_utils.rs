use std::{io::Write, fs};

use flate2::Compression;
use flate2::write::ZlibEncoder;
use sha1::{Sha1, Digest};



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

        let mut file_type = "";  
        if file_mode == "100644"{
            file_type = "blob";
        } else{
            file_type = "tree";
        }
        let file_path = file_atributes[1];

        let file_hash = file_atributes[2];

        println!("{} {} {} {}", file_mode, file_type, file_hash, file_path );
        
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

pub fn visit_dirs(dir: &std::path::Path) -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    let mut subfiles = visit_dirs(&path);
                    files.append(&mut subfiles);
                } else if let Some(file_name) = path.file_name() {
                    if let Some(name) = file_name.to_str() {
                        files.push(name.to_string());
                    }
                }
            }
        }
    }
    files
}
