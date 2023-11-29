use super::blob::Blob;
use super::tree::Tree;
use super::commit::Commit;
use super::tag::Tag;

#[derive(Debug)]

pub enum GitObject{
    Blob(Blob),
    Commit(Commit),
    Tree(Tree),
    Tag(Tag)
}