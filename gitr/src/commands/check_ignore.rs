use crate::file_manager::*;

fn check_ignore(paths: Vec<String>)->Result<Vec<(String,bool)>, GitError>{
    let gitignore = file_manager::read_file("gitignore")?;
    let lineas_ignore = gitignore.split("\n").collect();
    for path in paths{

    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test00_check_ignore_encuentra_gitignore(){
        assert_eq!(check_ignore("src/commands/check_ignore.rs"), true);
    }
}