extern crate flate2;

use std::io::Write;


use flate2::{Decompress, Compression};
use flate2::write::ZlibEncoder;
use crate::command_utils;
use crate::gitr_errors::{GitrError, self};
use crate::objects::git_object::GitObject;
use crate::objects::commit::Commit;
use crate::objects::tag::Tag;
use crate::objects::tree::Tree;
use crate::objects::blob::Blob;
use crate::git_transport::{ref_discovery::*,deltas::*};
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
    let (length, cursor) = get_encoded_length(data)?;
    let object_content = &data[cursor..];
    Ok((object_type, length, object_content, cursor))
}

fn get_encoded_length(data: &[u8]) -> Result<(usize,usize),GitrError> {
    let mut length = (data[0]<<4>>4) as usize;
    let mut cursor = 1;
    let mut shift = 4;
    while (data[cursor-1] & 0x80) != 0  {      
        length |= (data[cursor] as usize & 0x7F) << shift;
        shift += 7;
        if shift > 28 {
            return Err(GitrError::PackFileError("parse_git_object".to_string(),"La longitud es demasiado grande".to_string()));
        }
        cursor += 1;
    }
    Ok((length,cursor))
}

fn create_commit_object(decoded_data: &[u8])->Result<GitObject,GitrError>{
    let commit = Commit::new_commit_from_string(String::from_utf8_lossy(decoded_data).to_string())?;
    Ok(GitObject::Commit(commit))
}

fn create_tree_object(decoded_data: &[u8])->Result<GitObject,GitrError>{
    let tree = GitObject::Tree(Tree::new_from_packfile(decoded_data)?);
    Ok(tree)
}

fn create_tag_object(decoded_data: &[u8]) -> Result<GitObject,GitrError>{
    let tag = GitObject::Tag(Tag::new_tag_from_string(String::from_utf8_lossy(decoded_data).to_string())?);
    Ok(tag)
}

fn create_blob_object(decoded_data: &[u8])->Result<GitObject,GitrError>{
    let data_str = String::from_utf8_lossy(decoded_data);
    let blob = GitObject::Blob(Blob::new(data_str.to_string())?);

    Ok(blob)
}

fn git_valid_object_from_packfile(object_type: u8, decoded_data: &[u8],pack: &[u8],offset: usize)->Result<GitObject,GitrError>{
    let object = match  object_type{
        1 => create_commit_object(decoded_data)?,
        2 => create_tree_object(decoded_data)?,
        3 => create_blob_object(decoded_data)?,
        4 => create_tag_object(decoded_data)?,
        6 => transform_delta(decoded_data.to_vec(),pack.to_vec(),offset)?,
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
    let mut objects = vec![];

    let mut index: usize = 12;
    for _i in 0..num_objects {
        let (obj,leidos) = read_object(&mut buffer[index..])?;
        index += leidos;
        objects.push(obj);
    }
    Ok(objects)
}

pub fn read_object(buffer: &mut[u8]) -> Result<(GitObject,usize),GitrError>{
    match parse_git_object(buffer) {
        Ok((object_type, _length, mut object_content,cursor)) => {
            let mut ofs_base: usize = 0;
            if object_type == 6 {
                (ofs_base,object_content) = get_offset(object_content)?;
                let (_length,cursor1 ) = get_encoded_length(object_content)?;
                let (_length,cursor2 ) = get_encoded_length(&object_content[cursor1..])?;
                object_content = &object_content[cursor1+cursor2..];
            }
            let (decodeado, leidos) = decode(object_content).unwrap();
            let obj = git_valid_object_from_packfile(object_type, &decodeado[..],&buffer,ofs_base)?;
            let index = leidos as usize + cursor;
            return Ok((obj,index));
        },
        Err(err) => {
            println!("Error: {}", err);
            return Err(GitrError::PackFileError("read_pack_file".to_string(),"no se pudo parsear el objeto".to_string()));
        }
    }
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
        let (obj_type, obj_len) = h_str.split_once(" ").unwrap_or(("", ""));
        let (_, raw_data) = raw_data.split_at(1);
        contents.push((obj_type.to_string(), obj_len.to_string(), raw_data.to_vec()));
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
            "tag" => 4,
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
        if buffer.len() < 32 {
            println!("Error: No hay suficientes bytes para el packfile mínimo, se recibió {buffer:?}");
            return Err(GitrError::PackFileError("new_from_server_packfile".to_string(),"No hay suficientes bytes para el encabezado mínimo".to_string()));
        }
        verify_header(&buffer[..=3])?;
        let version = extract_version(&buffer[4..=7])?;
        let objects = read_pack_file(buffer)?;

        Ok(PackFile {
            _version: version,
            objects,})
    }

    
}