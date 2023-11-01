use std::mem;
use chrono::{Utc, Local, format};

use crate::gitr_errors::GitrError;
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
    pub fn new(tree: String, mut parent: String, author: String, committer: String, message: String) -> Result<Self, GitrError>{
        let mut format_data = String::new();
        let header = "commit ";
        


        let tree_format = format!("tree {}\n", tree);
        format_data.push_str(&tree_format);
        if parent != "None" {
            format_data.push_str(&format!("parent {}\n", parent));
        }
        parent = "".to_string();
        format_data.push_str(&format!("author {} <vschneider@fi.uba.ar> {} -0300\n", author, Utc::now().timestamp()));
        format_data.push_str(&format!("committer {} <vschneider@fi.uba.ar> {} -0300\n", committer, Utc::now().timestamp()));
        format_data.push_str("\n");
        format_data.push_str(&format!("{}\n", message));

        let size = format_data.as_bytes().len();

        let format_data_entera = format!("{}{}\0{}", header, size, format_data);

        let compressed_file = flate2compress(format_data_entera.clone())?;
        let hashed_file = sha1hashing(format_data_entera.clone());
        let hashed_file_str = hashed_file.iter().map(|b| format!("{:02x}", b)).collect::<String>();

        Ok(Commit {data:compressed_file,hash: hashed_file_str, tree, parent, author, committer, message })
    }

    pub fn save(&self) -> Result<(), GitrError>{
        crate::file_manager::write_object(self.data.clone(), self.hash.clone())?;
        Ok(())
    }

    pub fn get_hash(&self) -> String{
        self.hash.clone()
    }
}


/*
tree 4a2fe10e5e62c3d2b3a738d78df708f5b08af7af
parent 6e7c471ac3d96bf69e5a81b57a477401a6a4a6ea
author valen1611 <vschneider@fi.uba.ar> 1698605542 -0300
committer valen1611 <vschneider@fi.uba.ar> 1698605542 -0300

pre commit ahora si
*/