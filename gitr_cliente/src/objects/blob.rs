use crate::{file_manager, commands::commands::{flate2compress, sha1hashing}};
use std::error::Error;
use crate::objects::tree::Tree;

pub enum TreeEntry {
    Blob(Blob),
    Tree(Tree),
}
pub struct Blob{
    data: Vec<u8>,
    hash: String,
}

impl Blob{
    pub fn new(data: String) -> Result<Self, Box<dyn Error>>{
        let format_data = format!("blob {}\0{}", data.len(), data);
        let compressed_file = flate2compress(format_data)?;
        let hashed_file = sha1hashing(compressed_file);
        let hashed_file_str = hashed_file.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        Ok(Blob { data:compressed_file, hash:hashed_file_str })
    }
    pub fn save(&self) -> Result<(), Box<dyn Error>>{
        file_manager::write_object(self.data.clone(), self.hash)?;
        Ok(())
    }

    pub fn hash(&self) -> String{
        self.hash.clone()
    }
}