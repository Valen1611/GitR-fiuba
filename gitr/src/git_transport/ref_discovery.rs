use std::collections::HashSet;
use crate::{gitr_errors::{GitrError, self}, file_manager};

pub fn verify_header(header_slice: &[u8])->Result<(),GitrError>{
    let str_received = String::from_utf8_lossy(header_slice);
    if str_received != "PACK"{
        return Err(GitrError::PackFileError("verify_header".to_string(), "La signature no es PACK".to_string()));
    }
    Ok(())
}

pub fn extract_version(version_slice:&[u8])->Result<u32,GitrError>{
    let version = match version_slice.try_into(){
        Ok(vec) => vec,
        Err(_e) => return Err(gitr_errors::GitrError::PackFileError("extract_version".to_string(),"no se pudo obtener la version".to_string()))
    };
    let version = u32::from_be_bytes(version);
    Ok(version)
}

fn extract_head_hash(head_slice: &str)->String{
    let head_hash = head_slice.split(' ').collect::<Vec<&str>>()[0];
    head_hash.to_string().split_off(4)
}

fn extract_hash_and_ref(ref_slice: &str)->(String,String){
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
    if received_data == "0000"{
        return Ok(references)
    }
    let head_hash = extract_head_hash(iter_refs[0]);
    references.push((head_hash,"HEAD".to_string()));
    
    for refs in &iter_refs[1..]{
        if refs.is_empty() || *refs == "0000"{
            break;
        }
        references.push(extract_hash_and_ref(refs));
    }
    Ok(references)
}

pub fn reference_update_request(hash_n_references: Vec<(String,String)>, heads_ids: Vec<String>, heads_refs: Vec<String>)->Result<(String,bool,Vec<String>),GitrError>{
    let mut request = String::new();
    let mut j = 0;
    let mut pkt_needed = false;
    let mut pkt_ids:Vec<String> = vec![];
    for refer in heads_refs { // veo si tengo que crear o modificar alguna
        let mut falta = true;
        for hash_n_ref in hash_n_references.clone() {
            if hash_n_ref.1.ends_with(&refer) { 
                falta = false;
                if hash_n_ref.0 != heads_ids[j]{
                    pkt_needed = true;
                    pkt_ids.push(heads_ids[j].clone());
                    let line = format!("{} {} {}\n",hash_n_ref.0,heads_ids[j],hash_n_ref.1);
                    request.push_str(&format!("{:04X}{}",line.len()+4,line));
                }
                break; 
            }
        }
        if falta {
            pkt_needed = true;
            let mut ya_lo_tiene = false;
            for hash_n_ref in hash_n_references.clone() {
                if heads_ids[j] == hash_n_ref.0 {
                    ya_lo_tiene = true;
                    break;
                }
            }
            if !ya_lo_tiene {
                pkt_ids.push(heads_ids[j].clone());
            }
            let line = format!("0000000000000000000000000000000000000000 {} refs/heads/{}\n",heads_ids[j],refer);
            request.push_str(&format!("{:04X}{}",line.len()+4,line));
        }
        j += 1;
    }

    request.push_str("0000");
    Ok((request,pkt_needed,pkt_ids))
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
    if want_message == "0000"{
        return Ok(want_message.to_string());
    }
    if !client_commits.len() == 0{
        for have in file_manager::get_all_objects_hashes()? {
            let have_line = format!("have {}\n",have);
            want_message.push_str(&format!("{:04X}{}\n",have_line.len()+4,have_line));
        }
        
        want_message.push_str("0000");
    }
    want_message.push_str("0009done\n");
    Ok(want_message)
}