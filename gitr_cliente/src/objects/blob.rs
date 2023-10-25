use crate::file_manager;
use crate::gitr_errors::GitrError;
use std::error::Error;
use crate::objects::tree::Tree;
use crate::command_utils::{flate2compress, sha1hashing};
pub enum TreeEntry {
    Blob(Blob),
    Tree(Tree),
}
pub struct Blob{
    compressed_data: Vec<u8>,
    hash: String,
}

impl Blob{
    /// Pre: raw_data es el string que conteine el codigo
    /// 
    /// Post: Devuelve un Blob con el codigo comprimido y el hash
    pub fn new(raw_data: String) -> Result<Self, GitrError>{
        let format_data = format!("blob {}\0{}", raw_data.len(), raw_data);
        
        let compressed_data = flate2compress(format_data.clone())?;
        let hashed_file = sha1hashing(format_data);
        let hashed_file_str = hashed_file.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        Ok(Blob { 
            compressed_data:compressed_data, 
            hash:hashed_file_str
        })
    }
    pub fn save(&self) -> Result<(),GitrError>{
        file_manager::write_object(self.compressed_data.clone(), self.get_hash())?;
        Ok(())
    }

    pub fn get_hash(&self) -> String{
        self.hash.clone()
    }
}

