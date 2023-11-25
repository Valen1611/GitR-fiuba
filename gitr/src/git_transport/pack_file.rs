extern crate flate2;

use std::io::{Write};


use flate2::{Decompress, Compression};
use flate2::write::ZlibEncoder;
use crate::command_utils;
use crate::gitr_errors::{GitrError, self};
use crate::objects::git_object::GitObject;
use crate::objects::commit::Commit;
use crate::objects::tree::Tree;
use crate::objects::blob::Blob;
use crate::git_transport::ref_discovery::*;
#[derive(Debug)]
pub struct PackFile{
    _version: u32,
    pub objects: Vec<GitObject>,
}

fn decode(input: &[u8]) -> Result<(Vec<u8>,u64), std::io::Error> {
    let mut decoder = Decompress::new(true);
    let mut output:[u8; 1024] = [0;1024];
    decoder.decompress(input, &mut output, flate2::FlushDecompress::Finish)?;
    let cant_leidos = decoder.total_in();
    let output_return = output[..decoder.total_out() as usize].to_vec();
    
    Ok((output_return, cant_leidos))
}

fn code(input: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(input)?;
    encoder.finish()
}

fn parse_git_object(data: &[u8]) -> Result<(u8, usize, &[u8],usize), GitrError> {
    if data.len() < 2 {
        return Err(GitrError::PackFileError("parse_git_object".to_string(),"No hay suficientes bytes para el encabezado mínimo".to_string()));
    }
    let object_type = (data[0] << 1 >> 5) & 0x07;
    let mut length = (data[0]<<4>>4) as usize;
    let mut cursor = 1;
    let mut shift = 4;
    while (data[cursor-1] & 0x80) != 0 {        
        length |= (data[cursor] as usize & 0x7F) << shift;
        cursor += 1;
        shift += 7;
        if shift > 28 {
            return Err(GitrError::PackFileError("parse_git_object".to_string(),"La longitud es demasiado grande".to_string()));
        }
    }
    let object_content = &data[cursor..];
    Ok((object_type, length, object_content, cursor))
}

fn create_commit_object(decoded_data: &[u8])->Result<GitObject,GitrError>{
    let data_str = String::from_utf8_lossy(decoded_data);
    let data_for_commit = data_str.split('\n').collect::<Vec<&str>>();
    let (mut parent, mut tree, mut author, mut committer, mut message) = ("None","None","None","None","None");

    for line in data_for_commit{
        if line.starts_with("tree"){
            tree = line.split(' ').collect::<Vec<&str>>()[1];
        }
        if line.starts_with("parent"){
            parent = line.split(' ').collect::<Vec<&str>>()[1];
        }
        if line.starts_with("author"){
            author = match line.split_once(' '){
                Some(tupla) => tupla.1,
                None => return Err(GitrError::PackFileError("create_commit_object".to_string(),"Error al parsear el author".to_string()))
            };
        }
        if line.starts_with("committer"){
            committer = match line.split_once(' '){
                Some(tupla) => tupla.1,
                None => return Err(GitrError::PackFileError("create_commit_object".to_string(),"Error al parsear el commiter".to_string()))
            };
        }
        if line.contains('\0') || line.is_empty(){
            continue;
        }
        message = line;
    }
    
    let message_bien = "\n".to_owned()+message;

    let commit = GitObject::Commit(Commit::new_from_packfile(tree.to_string(), vec![parent.to_string()], author.to_string(), committer.to_string(), message_bien.to_string()).unwrap());
    Ok(commit)
}

fn create_tree_object(decoded_data: &[u8])->Result<GitObject,GitrError>{
    let tree = GitObject::Tree(Tree::new_from_packfile(decoded_data)?);
    Ok(tree)
}

fn create_blob_object(decoded_data: &[u8])->Result<GitObject,GitrError>{
    let data_str = String::from_utf8_lossy(decoded_data);
    let blob = GitObject::Blob(Blob::new(data_str.to_string())?);

    Ok(blob)
}

fn git_valid_object_from_packfile(object_type: u8, decoded_data: &[u8])->Result<GitObject,GitrError>{
    let object = match  object_type{
        1 => create_commit_object(decoded_data)?,
        2 => create_tree_object(decoded_data)?,
        3 => create_blob_object(decoded_data)?,
        _ => return Err(GitrError::PackFileError("git_valid_object_from_packfile".to_string(),"Tipo de objeto no válido".to_string()))
    };
    Ok(object)
}

pub fn read_pack_file(buffer: &mut[u8]) -> Result<Vec<GitObject>, GitrError> {
    let num_objects = match buffer[8..12].try_into(){
        Ok(vec) => vec,
        Err(_e) => return Err(gitr_errors::GitrError::PackFileError("read_pack_file".to_string(),"no se pudo obtener la # objetos".to_string()))
    };
    let num_objects = u32::from_be_bytes(num_objects);
    println!("deberian entrar {num_objects} objetos");
    let mut objects = vec![];

    let mut index: usize = 0;
    for _i in 0..num_objects {
        match parse_git_object(&buffer[12+index..]) {
            Ok((object_type, _length, object_content,cursor)) => {
                let (decodeado, leidos) = decode(object_content).unwrap();
                objects.push(git_valid_object_from_packfile(object_type, &decodeado[..])?);
                index += leidos as usize + cursor;
            }
            Err(err) => {
                println!("Error: {}", err);
                return Err(GitrError::PackFileError("read_pack_file".to_string(),"no se pudo parsear el objeto".to_string()));
            }
        }
    }
    Ok(objects)
}

pub fn prepare_contents(datos: Vec<Vec<u8>>) -> Vec<(String,String,Vec<u8>)> {
    let mut contents: Vec<(String, String, Vec<u8>)> = Vec::new();
    for data in datos {
        let mut i: usize = 0;
        for byte in data.clone() {
            if byte == b'\0' {
                break;
            }
            i += 1;
        }
        let (header, raw_data) = data.split_at(i);
        let h_str = String::from_utf8_lossy(header).to_string();
        let div = h_str.split(' ').collect::<Vec<&str>>();
        let (obj_type, obj_len) = (div[0].to_string(), div[1].to_string());
        let (_, raw_data) = raw_data.split_at(1);
        contents.push((obj_type, obj_len, raw_data.to_vec()));
    }
    contents
}

/// Recibe vector de strings con los objetos a comprimir y devuelve un vector de bytes con el packfile
pub fn create_packfile(contents: Vec<(String,String,Vec<u8>)>) -> Result<Vec<u8>,GitrError> {
    // ########## HEADER ##########
    let mut final_data: Vec<u8> = Vec::new();
    let header = "PACK".to_string();
    final_data.extend(header.as_bytes());
    let cant_bytes = contents.len().to_be_bytes();
    let ver: u32 = 2;
    final_data.extend(&ver.to_be_bytes());
    final_data.extend(&cant_bytes[4..8]);
    // ########## OBJECTS ##########
    for (obj_type,len, raw_data) in contents {
        let mut obj_data: Vec<u8> = Vec::new();
        let obj_type: u8 = match obj_type.as_str(){ // obtengo el tipo de objeto
            "commit" => 1,
            "tree" => 2,
            "blob" => 3,
            _ => return Err(GitrError::PackFileError("create_packfile".to_string(),"Tipo de objeto no válido".to_string()))
        };
        
        let obj_len = match len.parse::<usize>() { // obtengo la longitud del objeto
            Ok(len) => len,
            Err(_e) => return Err(GitrError::PackFileError("create_packfile".to_string(),"Longitud de objeto no válida".to_string()))
        };
        if obj_len < 16 {
            obj_data.push((obj_type << 4) | obj_len as u8);
        } else {
            // ###### SIZE ENCODING ######
            let mut size = obj_len;
            let mut size_bytes: Vec<u8> = Vec::new();
            size_bytes.push((obj_type << 4) | (size & 0x0F) as u8 | 0x80); // meto el tipo de objeto y los primeros 4 bits de la longitud
            size >>= 4;
            while size >= 128 {
                size_bytes.push((size & 0x7F) as u8 | 0x80); // meto los siguientes 7 bits de la longitud con un 1 adelante
                size >>= 7;
            }
            size_bytes.push(size as u8); // meto los últimos ultimos 7 bits de la longitud con un 0 adelante
            obj_data.extend(size_bytes);
        }  
        let compressed = match code(&raw_data) {
            Ok(compressed) => compressed,
            Err(_e) => return Err(GitrError::PackFileError("create_packfile".to_string(),"Error al comprimir el objeto".to_string()))
        };
        obj_data.extend(compressed);
        final_data.extend(obj_data); 
    }
    
    // ########## CHECKSUM ##########
    let hasheado = command_utils::sha1hashing2(final_data.clone());
    final_data.extend(&hasheado);


    Ok(final_data)
}



impl PackFile{
    pub fn new_from_server_packfile(buffer: &mut[u8])->Result<PackFile, GitrError>{
        verify_header(&buffer[..=3])?;
        let version = extract_version(&buffer[4..=7])?;
        let objects = read_pack_file(buffer)?;

        Ok(PackFile {
            _version: version,
            objects,})
    }

    
}