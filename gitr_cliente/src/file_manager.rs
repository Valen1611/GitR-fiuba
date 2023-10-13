use std::fs::File;
use std::io::prelude::*;
use std::fs;
use crate::gitr_errors::GitrError;




pub fn init_repository(name: &String) ->  Result<(),GitrError>{
        create_directory(name)?;
        create_directory(&(name.clone() + "/gitr"))?;
        create_directory(&(name.clone() + "/gitr/objects"))?;
        create_directory(&(name.clone() + "/gitr/refs"))?;
        create_directory(&(name.clone() + "/gitr/refs/heads"))?;
    
    Ok(())
}

fn create_directory(path: &String)->Result<(), GitrError>{
    match fs::create_dir(path){
        Ok(_) => Ok(()),
        Err(_) => Err(GitrError::DirectoryCreationError)
    }
}

pub fn write_object(hashed_file:String) -> Result<(), GitrError>{
    let folder_name = hashed_file[0..2].to_string();
    let file_name = hashed_file[2..].to_string();
    let dir = String::from("test1/gitr/objects/");
    let folder_dir = dir.clone() + &folder_name;

    create_directory(&folder_dir)?;
    write_file(dir + &folder_name +  "/" + &file_name, "".into())?;
    Ok(())
}


fn write_file(path: String, text: String) -> Result<(), GitrError> {
    let mut archivo = match File::create(&path) {
        Ok(archivo) => archivo,
        Err(_) => return Err(GitrError::FileCreationError(path)),
    };

    match archivo.write_all(text.as_bytes()) {
        Ok(_) => Ok(()),
        Err(_) => Err(GitrError::FileWriteError(path)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_repository(){
        let test1= String::from("test1");
        assert!(init_repository(&test1).is_ok());

    }
}