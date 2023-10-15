use std::fs::File;
use std::io::prelude::*;
use std::fs;
use crate::gitr_errors::GitrError;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;


fn write_compressed_data(path: &str, data: &[u8]) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut encoder = ZlibEncoder::new(file, Compression::default());
    encoder.write_all(data)?;
    encoder.finish()?;
    Ok(())
}

fn read_compressed_file(path: &str) -> std::io::Result<Vec<u8>> {
    let file = File::open(path)?;
    let mut decoder = ZlibDecoder::new(file);
    let mut buffer = Vec::new();
    decoder.read_to_end(&mut buffer)?;
    Ok(buffer)
}

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

pub fn write_object(data:Vec<u8>, hashed_file:String) -> Result<(), GitrError>{
    let folder_name = hashed_file[0..2].to_string();
    let file_name = hashed_file[2..].to_string();
    let dir = String::from("gitr/objects/");
    let folder_dir = dir.clone() + &folder_name;

    create_directory(&folder_dir)?;
    write_compressed_data(&folder_dir, &data);
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

pub fn read_object(object: &String) -> Result<String, GitrError>{
    let folder_name = object[0..2].to_string();
    let file_name = object[2..].to_string();
    let dir = String::from("gitr/objects/");
    let folder_dir = dir.clone() + &folder_name;
    let path = dir + &folder_name +  "/" + &file_name;
    if !fs::metadata(&folder_dir).is_ok(){
        return Err(GitrError::ObjectNotFound);
    }
    if !fs::metadata(&path).is_ok(){
        return Err(GitrError::ObjectNotFound);
    }
    let data = read_compressed_file(&path);
    let data = match data{
        Ok(data) => data,
        Err(_) => return Err(GitrError::FileReadError(path)),
    };
    let data = String::from_utf8(data);
    let data = match data{
        Ok(data) => data,
        Err(_) => return Err(GitrError::FileReadError(path)),
    };
    Ok(data)
    
    
    

    

    

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