use std::collections::HashSet;

use chrono::Utc;

use crate::file_manager::{self, get_object};
use crate::gitr_errors::GitrError;
use crate::command_utils::{flate2compress, sha1hashing, get_user_mail_from_config};

use super::tree::Tree;

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
        format_data.push_str(&format!("author {} <{}> {} -0300\n", author, get_user_mail_from_config()?, Utc::now().timestamp()));
        format_data.push_str(&format!("committer {} <{}> {} -0300\n", committer, get_user_mail_from_config()?, Utc::now().timestamp()));
        format_data.push_str("\n");
        let message = message.replace("\"", "");
        format_data.push_str(&format!("{}\n", message));
        let size = format_data.as_bytes().len();
        let format_data_entera = format!("{}{}\0{}", header, size, format_data);
        let compressed_file = flate2compress(format_data_entera.clone())?;
        let hashed_file = sha1hashing(format_data_entera.clone());
        let hashed_file_str = hashed_file.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        Ok(Commit {data:compressed_file,hash: hashed_file_str, tree, parent, author, committer, message })
    }

    pub fn new_from_packfile(tree: String, mut parent: String, author: String, committer: String, message: String) -> Result<Self, GitrError>{
        let mut format_data = String::new();
        let header = "commit ";
        let tree_format = format!("tree {}\n", tree);
        format_data.push_str(&tree_format);
        if parent != "None" {
            format_data.push_str(&format!("parent {}\n", parent));
        }
        format_data.push_str(&format!("author {}\n", author)); //Utc::now().timestamp()
        format_data.push_str(&format!("committer {}", committer));
        format_data.push_str("\n");
        format_data.push_str(&format!("{}\n", message));
        let size = format_data.as_bytes().len();      
        let format_data_entera = format!("{}{}\0{}", header, size, format_data);
        let compressed_file = flate2compress(format_data_entera.clone())?;
        let hashed_file = sha1hashing(format_data_entera.clone());
        let hashed_file_str = hashed_file.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        Ok(Commit {data:compressed_file,hash: hashed_file_str, tree, parent, author, committer, message })
    }

    pub fn save(&self,cliente: String) -> Result<(), GitrError>{
        crate::file_manager::write_object(self.data.clone(), self.hash.clone(),cliente)?;
        Ok(())
    }

    pub fn get_hash(&self) -> String{
        self.hash.clone()
    }

    pub fn get_data(&self) -> Vec<u8>{
        self.data.clone()
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

    pub fn get_objects_from_commits(commits_id: Vec<String>,client_objects: Vec<String>, r_path: String) -> Result<Vec<String>,GitrError> {
        // Voy metiendo en el objects todo lo que no haya que mandarle denuevo al cliente
        let mut object_ids: HashSet<String> = HashSet::new();
        for obj_id in client_objects.clone() {
            object_ids.insert(obj_id.clone());
        }
        let mut commits: Vec<Commit> = Vec::new();
        for id in commits_id {
            
            object_ids.insert(id.clone());
            match Commit::new_commit_from_data(file_manager::get_object(id, r_path.clone())?) {
                Ok(commit) => {commits.push(commit)},
                _ => {return Err(GitrError::InvalidCommitError)}
            }
        } 
        println!("commits len {}",commits.len());
        for commit in commits {
            println!("===commit: {}\n",String::from_utf8_lossy(&file_manager::decode(&commit.get_data()).unwrap()));
            object_ids.insert(commit.get_tree());

            Tree::get_all_tree_objects(commit.get_tree(), r_path.clone(), &mut object_ids)?;
        }
        for obj in client_objects{
            object_ids.remove(&obj);
        } 
        Ok(Vec::from_iter(object_ids.into_iter()))
    
        
    }

    pub fn get_parents(commits_ids: Vec<String>, receivers_commits: Vec<String>, r_path: String) -> Result<Vec<String>, GitrError>{
        let mut parents: Vec<String> = Vec::new();
        let mut rcv_commits = HashSet::new();
        for id in receivers_commits {
            rcv_commits.insert(id);
        }
        for id in commits_ids {
            if rcv_commits.contains(&id) {
                continue;
            } 
            parents.push(id.clone());
            match Commit::new_commit_from_data(file_manager::get_object(id, r_path.clone())?) {
                Ok(commit) => {Self::get_parents_rec(commit.parent, &rcv_commits,r_path.clone(),&mut parents)?},
                _ => {return Err(GitrError::InvalidCommitError)}
            }
        }
        Ok(Vec::from_iter(parents.into_iter()))
    }

    fn get_parents_rec(id: String, receivers_commits: &HashSet<String>,r_path: String, parents: &mut Vec<String>) -> Result<(), GitrError>{
        if receivers_commits.contains(&id) || id == "None" || id == ""{
            return Ok(());
        }
        parents.push(id.clone());
        match Commit::new_commit_from_data(file_manager::get_object(id, r_path.clone())?) {
            Ok(commit) => {Self::get_parents_rec(commit.parent, receivers_commits, r_path, parents)
            },
            _ => {return Err(GitrError::InvalidCommitError)}
        }
    }
}


#[cfg(test)]
mod tests {

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
