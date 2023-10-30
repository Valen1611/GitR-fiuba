use crate::gitr_errors::GitrError;
use crate::{file_manager, command_utils};
use crate::command_utils::{flate2compress, sha1hashing};
use super::blob::TreeEntry;

#[derive(Debug)]
pub struct Tree{
    entries: Vec<(String,TreeEntry)>,
    data: Vec<u8>,
    hash: String,
}



use to_binary::{BinaryString,BinaryError};

/*
commit -> tree -> src ->

*/
use std::fmt::{self, Write};
use std::fs::File;
use std::io::prelude::*;
use std::fs::*;


pub fn get_formated_hash(hash: String, path: &String) -> Result<Vec<u8>, GitrError>{
    let mut formated_hash:  Vec<u8> = Vec::new();
    for i in (0..40).step_by(2) {
        let first_char = hash.as_bytes()[i] as char;
        let second_char = hash.as_bytes()[i+1] as char;
        let byte_as_str = format!("{}{}", first_char, second_char);
        let byte = match u8::from_str_radix(&byte_as_str, 16)
        {
            Ok(byte) => byte,
            Err(_) => return Err(GitrError::FileReadError(path.clone())),
        };
        println!("byte: {:08b}", byte);
        let compressed_byte = match command_utils::flate2compress2(vec![byte]) {
            Ok(byte) => byte,
            Err(_) => return Err(GitrError::CompressionError),
        };
        formated_hash.push(byte);
    }
    Ok(formated_hash)
}

impl Tree{
    pub fn new(entries: Vec<(String,TreeEntry)>) ->  Result<Self, GitrError>{
        
        let mut objs_entries = Vec::new();
        let mut entries_size: u8 = 0;
        for (path, entry) in &entries {
            match entry {
                TreeEntry::Blob(blob) => {
                    let hash = blob.get_hash();
                    let formated_hash = get_formated_hash(hash, path)?;

                    let path_no_repo = path.split_once('/').unwrap().1;

                    let obj_entry = vec! [
                        b"100644 ",
                        path_no_repo.as_bytes(),
                        b"\0",
                        &formated_hash,
                    ]
                    .concat();

                    entries_size += obj_entry.len() as u8;
                    objs_entries.push(obj_entry);                
                
                }
                TreeEntry::Tree(tree) => {
                    //objs_entries.push_str(&format!("40000 {}\0{}\n", path, tree.hash));
                }
            }
        }
        
    
        //        write!(&mut binary, "{:08b}", byte).unwrap();

        let mut data = vec![
            b"tree ",
            entries_size.to_string().as_bytes(),
            b"\0",
            &objs_entries.concat(),
        ].concat();


        println!("data: {:?}", data);

        let compressed_file2 = command_utils::flate2compress2(data.clone())?;
        let hashed_file2 = command_utils::sha1hashing2(data.clone());

        let hashed_file_str = hashed_file2.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        println!("HASHED FILE: {:?}", hashed_file_str);

        let mut format_data = String::new();
        let init = format!("tree {}\0", entries.len());
        format_data.push_str(&init);


        format_data = format_data.trim_end().to_string();
        //let compressed_file = flate2compress(data.clone())?;


        //let hashed_file = sha1hashing(data.clone());
        Ok(Tree {entries, data: compressed_file2, hash: hashed_file_str })
    }

    pub fn save(&self) -> Result<(), GitrError>{
        file_manager::write_object(self.data.clone(), self.hash.clone())?;
        Ok(())
    }
    
    pub fn get_hash(&self) -> String{
        self.hash.clone()
    }

    
}


#[cfg(test)]
mod tests {
    use crate::objects::blob::Blob;
    use super::*;

    // #[test]
    // fn tree_creation_test(){
    //     let blob = Blob::new("hola".to_string()).unwrap();
    //     blob.save().unwrap();
    //     let hash = blob.get_hash();
    //     let tree = Tree::new(vec![("hola.txt".to_string(), TreeEntry::Blob(blob))]).unwrap();
    //     tree.save().unwrap();
    // }
 
}
