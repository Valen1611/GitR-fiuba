use std::vec;

use crate::file_manager;
use crate::gitr_errors::GitrError;

fn armar_path(path: String, cliente: String)->Result<String,GitrError>{
    let full_path = vec![
        file_manager::get_current_repo(cliente)?,
        "/gitr/".to_string(),
        path
    ];
    Ok(full_path.concat())
}

pub fn check_ignore(paths: Vec<String>, client: String)->Result<Vec<String>, GitrError>{
    let gitignore = file_manager::read_file(armar_path("gitrignore".to_string(),client.clone())?)?;
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
        let cliente = "cliente".to_string();
        let gitignore = file_manager::read_file(armar_path("gitrignore".to_string(),cliente).unwrap());
        assert!(gitignore.is_ok());
    }

    #[test]
    fn test01_check_ignore_lee_correctamente_una_linea(){
        let cliente = "cliente".to_string();
        let paths = vec!["/target".to_string()];
        let vec_match = vec![
            "/target".to_string()
        ];
        assert_eq!(check_ignore(paths,cliente).unwrap(), vec_match);
    }
}