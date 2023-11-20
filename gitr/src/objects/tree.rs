use std::collections::HashSet;

use crate::gitr_errors::GitrError;
use crate::{file_manager, command_utils};
use super::blob::TreeEntry;

#[derive(Debug)]
pub struct Tree{
    entries: Vec<(String,TreeEntry)>,
    data: Vec<u8>,
    hash: String,
}


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
        formated_hash.push(byte);
    }
    Ok(formated_hash)
}

impl Tree{
    pub fn new(mut entries: Vec<(String,TreeEntry)>) ->  Result<Self, GitrError>{
        
        entries.sort_by(|a, b| a.0.cmp(&b.0));

        let mut objs_entries = Vec::new();
        let mut entries_size: u8 = 0;
        for (path, entry) in &entries {
            match entry {
                TreeEntry::Blob(blob) => {
                    let hash = blob.get_hash();
                    let formated_hash = get_formated_hash(hash, path)?;

                    let _path_no_repo = path.split_once('/').unwrap().1;
                    let file_name = path.split('/').last().unwrap();
                    let obj_entry = [
                        b"100644 ",
                        file_name.as_bytes(),
                        b"\0",
                        &formated_hash,
                    ]
                    .concat();

                    entries_size += obj_entry.len() as u8;
                    objs_entries.push(obj_entry);                
                
                }
                TreeEntry::Tree(tree) => {
                    let hash = tree.get_hash();
                    let formated_hash = get_formated_hash(hash, path)?;

                    let obj_entry = [
                        b"40000 ",
                        path.as_bytes(),
                        b"\0",
                        &formated_hash,
                    ]
                    .concat();

                    entries_size += obj_entry.len() as u8;
                    objs_entries.push(obj_entry);    
                }
            }
        }
        

        let data = [
            b"tree ",
            entries_size.to_string().as_bytes(),
            b"\0",
            &objs_entries.concat(),
        ].concat();

        let compressed_file2 = command_utils::flate2compress2(data.clone())?;
        let hashed_file2 = command_utils::sha1hashing2(data.clone());

        let hashed_file_str = hashed_file2.iter().map(|b| format!("{:02x}", b)).collect::<String>();

        let mut format_data = String::new();
        let init = format!("tree {}\0", entries.len());
        format_data.push_str(&init);
        Ok(Tree {entries, data: compressed_file2, hash: hashed_file_str })

        
    }

    pub fn new_from_packfile(raw_data: &[u8])->  Result<Self, GitrError>{
        let header_len = raw_data.len();
        let tree_raw_file = [b"tree ",
            header_len.to_string().as_bytes(),
            b"\0",
            raw_data].concat();

        let compressed_data = command_utils::flate2compress2(tree_raw_file.clone())?;
        let hash = command_utils::sha1hashing2(tree_raw_file.clone());
        let tree_hash = hash.iter().map(|b| format!("{:02x}", b)).collect::<String>();

        let tree = Tree{entries: vec![], data: compressed_data, hash: tree_hash};
        Ok(tree)
    }

    pub fn save(&self,cliente: String) -> Result<(), GitrError>{
        file_manager::write_object(self.data.clone(), self.hash.clone(),cliente)?;
        Ok(())
    }
    
    pub fn get_hash(&self) -> String{
        self.hash.clone()
    }
    
    pub fn get_data(&self) -> Vec<u8>{
        self.data.clone()
    }

    pub fn get_objects_id_from_string(data: String) -> Result<Vec<String>, GitrError> {
        
        if data.split_at(4).0 != "tree"{
            return Err(GitrError::InvalidTreeError);
        }
        
        let mut objects_id = Vec::new();
        
        let raw_data = match data.split_once('\0') {
            Some((_, raw_data)) => raw_data,
            None => {
                println!("Error: invalid object type");
                return Err(GitrError::InvalidTreeError)
            }
        };
        for entry in raw_data.split('\n'){
            let _new_path_hash = entry.split(' ').collect::<Vec<&str>>()[1];
            let hash = _new_path_hash.split('\0').collect::<Vec<&str>>()[1];
            objects_id.push(hash.to_string());
        }
        Ok(objects_id)
    }
            
          
    
    pub fn get_all_tree_objects(tree_id: String, r_path: String, object_ids: &mut HashSet<String>) -> Result<(),GitrError> {
        if let Ok(tree_str) = file_manager::read_object(&tree_id, r_path.clone(), false) {
            let tree_objects = match Tree::get_objects_id_from_string(tree_str){
                Ok(ids) => {ids},
                _ => return Err(GitrError::InvalidTreeError)
            };
            for obj_id in tree_objects {
                object_ids.insert(obj_id.clone());
                let _ = Self::get_all_tree_objects(obj_id.clone(), r_path.clone(),object_ids); 
            }

            return Ok(())
        }
        Err(GitrError::InvalidTreeError)
    }
    
}
