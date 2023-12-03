use std::vec;

use crate::file_manager;
use crate::gitr_errors::GitrError;

fn armar_path(path: String, cliente: String)->Result<String,GitrError>{
    let full_path = vec![
        file_manager::get_current_repo(cliente)?,
        "/".to_string(),
        path
    ];
    println!("full_path: {:?}", full_path);
    Ok(full_path.concat())
}

pub fn check_ignore(paths: Vec<String>, client: String)->Result<Vec<String>, GitrError>{
    let path = armar_path("gitrignore".to_string(),client.clone())?;
    println!("path: {}", path);
    let gitignore = file_manager::read_file(path)?;
    println!("gitignore: {}", gitignore);
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
    use std::{fs, path::Path};

    use crate::commands::commands;

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

    #[test]
    fn test02_check_ignore_lee_correctamente_varias_lineas_desde_archivo(){
        let path = Path::new(&"cliente");
        if path.exists() {
            fs::remove_dir_all(path).unwrap();
        }
        file_manager::create_directory(&"cliente".to_string()).unwrap();
        let cliente = "cliente".to_string();
        let flags = vec!["repo_ignore".to_string()];
        commands::init(flags, cliente.clone()).unwrap();
        file_manager::write_file("cliente/repo_ignore/gitrignore".to_string(), "/target\n/target2\n".to_string()).unwrap();
        let paths = vec!["/target".to_string(), "/target2".to_string()];
        let vec_match = vec![
            "/target".to_string(),
            "/target2".to_string()
        ];
        assert_eq!(check_ignore(paths,cliente).unwrap(), vec_match);
    }
}