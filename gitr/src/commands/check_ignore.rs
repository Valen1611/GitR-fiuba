use std::vec;

use crate::file_manager;
use crate::gitr_errors::GitrError;

fn armar_path(path: String)->Result<String,GitrError>{
    let full_path = vec![
        file_manager::get_current_repo()?,
        "/gitr/".to_string(),
        path
    ];
    Ok(full_path.concat())
}

pub fn check_ignore(paths: Vec<String>)->Result<Vec<String>, GitrError>{
    let gitignore = file_manager::read_file(armar_path("gitrignore".to_string())?)?;
    let lineas_ignore:Vec<&str> = gitignore.split("\n").collect();
    let mut ignored_paths = vec![];
    for path in paths{
        if lineas_ignore.contains(&path.as_str()){
            ignored_paths.push(path);
        }
    }
    Ok(ignored_paths)
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test00_check_ignore_encuentra_gitignore(){
        let paths = vec!["/target".to_string()];
        let vec_match = vec![
            "/target".to_string()
        ];
        assert_eq!(check_ignore(paths).unwrap(), vec_match);
    }
}