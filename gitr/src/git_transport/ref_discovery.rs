// La idea de este módulo es manejar el primer contacto con el servidor y la búsqueda de referencias para armar el directorio.

use std::collections::HashSet;

use crate::{gitr_errors::{GitrError, self}, file_manager};

pub fn verify_header(header_slice: &[u8])->Result<(),GitrError>{
    let str_received = String::from_utf8_lossy(header_slice);
    println!("verify_header(): str_received: {:?}", str_received);
    if str_received != "PACK"{
        return Err(GitrError::PackFileError("verify_header".to_string(), "La signature no es PACK".to_string()));
    }
    Ok(())
}

pub fn extract_version(version_slice:&[u8])->Result<u32,GitrError>{
    let version = match version_slice.try_into(){
        Ok(vec) => vec,
        Err(e) => return Err(gitr_errors::GitrError::PackFileError("extract_version".to_string(),"no se pudo obtener la version".to_string()))
    };
    let version = u32::from_be_bytes(version);
    println!("Versión del archivo de pack: {:?}", version);
    Ok(version)
}

fn extract_head_hash(head_slice: &str)->String{
    println!("extract_head_hash(): {}", head_slice);
    let head_hash = head_slice.split(' ').collect::<Vec<&str>>()[0];
    head_hash.to_string().split_off(4)
}

fn extract_hash_and_ref(ref_slice: &str)->(String,String){
    println!("extract_hash_and_ref(): [param:ref_slice]{:?}", ref_slice);
    let split = ref_slice.split(' ').collect::<Vec<&str>>();
    let hash = split[0];
    let reference = split[1];
    (hash.to_string().split_off(4), reference.to_string())
}

/// Devuelve Vector de tuplas (hash, referencia)
pub fn discover_references(received_data: String) -> Result<Vec<(String,String)>,GitrError>{
    let mut references: Vec<(String,String)> = vec![];
    let iter_refs: Vec<&str> = received_data.lines().collect();
    //Extraigo el primer hash al que apunta HEAD
    let head_hash = extract_head_hash(iter_refs[0]);
    references.push((head_hash,"HEAD".to_string()));
    
    for refs in &iter_refs[1..]{
        if *refs == ""{
            break;
        }
        references.push(extract_hash_and_ref(refs));
    }
    // println!("Pares hash - ref{:?}", references);
    Ok(references)
}

pub fn assemble_want_message(references: &Vec<(String,String)>, client_commits:Vec<String>)->Result<String,GitrError>{
    let set = client_commits.clone().into_iter().collect::<HashSet<String>>();
    let mut want_message = String::new();
    for refer in references{
        if set.contains(&refer.0){
            continue;
        }
        let want_line = format!("want {}\n",refer.0);
        want_message.push_str(&format!("{:04X}{}",want_line.len()+4,want_line));
    }
    want_message.push_str("0000");
    if !client_commits.len() == 0{
        for have in file_manager::get_all_objects()? {
            let have_line = format!("have {}\n",have);
            want_message.push_str(&format!("{:04X}{}\n",have_line.len()+4,have_line));
        }
        
        want_message.push_str("0000");
    }
    want_message.push_str("0009done\n");
    // println!("assemble_want_message(): {:?}", want_message);
    Ok(want_message)
}