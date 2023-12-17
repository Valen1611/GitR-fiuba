
use serde::{Serialize, Deserialize};

use crate::gitr_errors::GitrError;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PullRequest {
    pub id: u8,
    pub title: String,
    pub description: String,
    pub head: String, // branch name (feature)
    pub base: String, // branch name (master casi siempre)
    pub status: String, // open, closed
    //commits: Vec<String>,
}

impl PullRequest {
    pub fn new(
        id: u8,
        title: String,
        description: String,
        head: String,
        base: String,
        //commits: Vec<String>,
    ) -> Self {

        PullRequest {
            id,
            title,
            description,
            head,
            base,
            //commits,
            status: String::from("open"),
        }
    }

    pub fn merge_pr() {
        // que se mergee head en base
    }

    pub fn get_commits(&self) -> Vec<String> {
        let mut commits: Vec<String> = Vec::new();
        commits.push(String::from("commit1"));
        commits.push(String::from("commit2"));
        commits.push(String::from("commit3"));
        commits

    }

    pub fn to_string(&self) -> Result<String, GitrError> {
            
        match serde_json::to_string(&self) {
            Ok(json) => Ok(json),
            Err(_) => Err(GitrError::PullRequestWriteError),
        }
    }

    pub fn from_string(content: String) -> Result<Self, GitrError> {
        match serde_json::from_str(&content) {
            Ok(pr) => Ok(pr),
            Err(_) => Err(GitrError::PullRequestReadError),
        }    
    }

    pub fn get_branch_name(&self) -> &String {
        &self.head
    }

    pub fn get_base_name(&self) -> &String {
        &self.base
    }

    pub fn get_status(&self) -> &String {
        &self.status
    }

}