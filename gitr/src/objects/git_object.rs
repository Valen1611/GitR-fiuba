use super::blob::Blob;
use super::tree::Tree;
use super::commit::Commit;

#[derive(Debug)]

pub enum GitObject{
    Blob(Blob),
    Commit(Commit),
    Tree(Tree)
}