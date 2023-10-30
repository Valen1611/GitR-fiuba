use std::mem;
use chrono::{Utc, Local};

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
    pub fn new(tree: String, parent: String, author: String, committer: String, message: String) -> Result<Self, GitrError>{
        let mut format_data = String::new();
        let init = format!("commit {}\0",mem::size_of::<Self>());
        // agregar la palabra tree que falta
        // y fijarse como agarrar el tamanio
        format_data.push_str(&init);
        let tree_format = format!("tree {}\n", tree);
        format_data.push_str(&tree_format);
        format_data.push_str(&format!("parent {}\n", parent));
        format_data.push_str(&format!("author {} {} {} \n", author, Utc::now().timestamp(), Local::now().offset()));
        format_data.push_str(&format!("committer {}\n", committer));
        format_data.push_str("\n");
        format_data.push_str(&message);
        let compressed_file = flate2compress(format_data.clone())?;
        let hashed_file = sha1hashing(format_data.clone());
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
    pub fn get_tree(&self) -> String{
        self.tree.clone()
    }

    pub fn new_commit_from_string(data: String)->Result<Commit,GitrError>{
        let (mut parent, mut tree, mut author, mut committer, mut message) = ("","None","None","None","None");
        
        for line in data.lines() {
            let elems = line.split(" ").collect::<Vec<&str>>();
            
            match elems[0] {
                "tree" => tree = elems[1],
                "parent" => parent = elems[1],
                "author" => author = elems[1],
                "committer" => committer = elems[1],
                _ => message = line,
            }
        }
        
        let commit = Commit::new(tree.to_string(), parent.to_string(), author.to_string(), committer.to_string(), message.to_string())?;
        Ok(commit)
    }

    pub fn new_commit_from_data(data: String) -> Result<Commit, GitrError>{
       let commit_string = data.split("\0").collect::<Vec<&str>>()[1].to_string();
       Ok(Self::new_commit_from_string(commit_string)?)
    }
}


#[cfg(test)]
mod tests {
    use std::mem;

    use crate::objects::commit::Commit;
    #[test]
    fn test01_new_commit_from_string() {

        let commit = Commit::new("tree".to_string(), "parent".to_string(), "author".to_string(), "committer".to_string(), "message".to_string()).unwrap();
        let commit_string = format!("tree {}\nparent {}\nauthor {} {} {}\ncommitter {}\n\nmessage", commit.tree, commit.parent, commit.author, "timestamp", "Buenos Aires +3", commit.committer);
        let commit_from_string = Commit::new_commit_from_string(commit_string).unwrap();
        assert_eq!(commit_from_string.tree, commit.tree);
        assert_eq!(commit_from_string.parent, commit.parent);
        assert_eq!(commit_from_string.author, commit.author);
        assert_eq!(commit_from_string.committer, commit.committer);
        assert_eq!(commit_from_string.message, commit.message);
    }

    #[test]
    fn new_commit_from_data() {
        let commit = Commit::new("tree".to_string(), "parent".to_string(), "author".to_string(), "committer".to_string(), "message".to_string()).unwrap();
        let commit_string = format!("commit <lenght>\0tree {}\nparent {}\nauthor {} {} {}\ncommitter {}\n\nmessage", commit.tree, commit.parent, commit.author, "timestamp", "Buenos Aires +3", commit.committer);
        let commit_from_string = Commit::new_commit_from_data(commit_string).unwrap();
        assert_eq!(commit_from_string.tree, commit.tree);
        assert_eq!(commit_from_string.parent, commit.parent);
        assert_eq!(commit_from_string.author, commit.author);
        assert_eq!(commit_from_string.committer, commit.committer);
        assert_eq!(commit_from_string.message, commit.message);
    }
}