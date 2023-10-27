use std::{mem, error::Error};

use crate::command_utils::{flate2compress, sha1hashing};

#[derive(Debug)]
pub struct Commit{
    data: Vec<u8>,
    hash: String,
    tree: String,
    parent: String,
    author: String,
    committer: String,
    message: String,
}

impl Commit{
    pub fn new(tree: String, parent: String, author: String, committer: String, message: String) -> Result<Self, Box<dyn Error>>{
        let mut format_data = String::new();
        let init = format!("commit {}\0",mem::size_of::<Self>());
        format_data.push_str(&init);
        format_data.push_str(&tree);
        format_data.push_str("\n");
        format_data.push_str(&format!("author {}\n", author));
        format_data.push_str(&format!("committer {}\n", committer));
        format_data.push_str("\n");
        format_data.push_str(&message);
        let compressed_file = flate2compress(format_data.clone())?;
        let hashed_file = sha1hashing(format_data.clone());
        let hashed_file_str = hashed_file.iter().map(|b| format!("{:02x}", b)).collect::<String>();

        Ok(Commit {data:compressed_file,hash: hashed_file_str, tree, parent, author, committer, message })

    }

    pub fn save(&self) -> Result<(), Box<dyn Error>>{
        crate::file_manager::write_object(self.data.clone(), self.hash.clone())?;
        Ok(())
    }

    pub fn get_hash(&self) -> String{
        self.hash.clone()
    }
}