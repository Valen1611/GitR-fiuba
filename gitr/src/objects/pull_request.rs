
use serde::{Serialize, Deserialize};


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
}