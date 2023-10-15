use std::error::Error;

use crate::{file_manager, commands::commands::{flate2compress, sha1hashing}};

use super::blob::TreeEntry;


pub struct Tree{
    entries: Vec<(String,TreeEntry)>,
    data: Vec<u8>,
    hash: String,
}

impl Tree{
    pub fn new(entries: Vec<(String,TreeEntry)>) ->  Result<Self, Box<dyn Error>>{
        let mut format_data = String::new();
        let init = format!("tree {}\0", entries.len());
        format_data.push_str(&init);
        for (path, entry) in &entries {
            match entry {
                TreeEntry::Blob(blob) => {
                    format_data.push_str(&format!("100644 {} {}\0", path, blob.hash()));
                }
                TreeEntry::Tree(tree) => {
                    format_data.push_str(&format!("40000 {} {}\0", path, tree.hash));
                }
            }
        }
        let compressed_file = flate2compress(format_data)?;
        let hashed_file = sha1hashing(compressed_file);
        let hashed_file_str = hashed_file.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        Ok(Tree { entries:entries, data: compressed_file, hash: hashed_file_str })
    }

    pub fn save(&self) -> Result<(), Box<dyn Error>>{
        file_manager::write_object(self.data.clone(), self.hash.clone())?;
        Ok(())
    }
    

    
}