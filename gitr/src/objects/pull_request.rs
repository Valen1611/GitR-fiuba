
use serde::{Serialize, Deserialize};

use crate::gitr_errors::GitrError;


#[derive(Serialize, Deserialize, Debug)]
pub struct PullRequest {
    pub id: u8,
    pub title: String,
    pub description: String,
    pub head: String, // branch name (feature)
    pub base: String, // branch name (master casi siempre)
    status: String, // open, closed, merged
    commits: Vec<String>,
}

impl PullRequest {
    pub fn new(
        id: u8,
        title: String,
        description: String,
        head: String,
        base: String,
        commits: Vec<String>,
    ) -> Self {

        PullRequest {
            id,
            title,
            description,
            head,
            base,
            commits,
            status: String::from("open"),
        }
    }

    pub fn merge_pr() {
        // que se mergee head en base
    }

    pub fn get_commits(&self) -> Vec<String> {
        self.commits.clone()
    }

    pub fn to_string(&self) -> Result<String, GitrError> {
            
        match serde_json::to_string(&self) {
            Ok(json) => Ok(json),
            Err(_) => Err(GitrError::PullRequestWriteError),
        }
    }
}