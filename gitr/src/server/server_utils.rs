extern crate flate2;
use std::collections::HashSet;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Error;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::str::from_utf8;

use std::thread;
use crate::file_manager;
use crate::git_transport::pack_file::PackFile;
use crate::git_transport::pack_file::create_packfile;
use crate::git_transport::pack_file::prepare_contents;

use crate::git_transport::ref_discovery;
use crate::objects::commit::Commit;
use crate::objects::git_object::GitObject;


pub fn server_init (s_addr: &str) -> std::io::Result<()>  {
    let listener = TcpListener::bind(s_addr)?;
    let mut childs = Vec::new();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {                
                let builder = thread::Builder::new().name("cliente_random".to_string());

                childs.push(builder.spawn(|| {handle_client(stream)})?);
            }
            Err(e) => {
                eprintln!("Error al aceptar la conexión: {}", e);
            }
        }
    }
    for child in childs {
        match child.join(){
            Ok(result) => {result?},
            Err(_e) => {return Err(Error::new(std::io::ErrorKind::Other, "Error en alguno de los hilos"))}
        }
    }
    Ok(())
}

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {

    let mut buffer = [0; 1024];
    let guardados_id: HashSet<String>;
    let refs_string :String;

    if let Ok(n) = stream.read(&mut buffer) {
        if n == 0 {
            return Ok(());
        }
        
        // ########## HANDSHAKE ##########
        let pkt_line = String::from_utf8_lossy(&buffer[..n]).to_string(); 
        match is_valid_pkt_line(&pkt_line) {
            Ok(_) => {},
            Err(_) => {let _ = stream.write("Error: no se respeta el formato pkt-line".as_bytes());
            return Ok(())}
        }
        let elems = split_n_validate_elems(&pkt_line)?;
        let r_path = elems[2].to_string();
        let _ = create_dirs(&r_path);
        // ########## REFERENCE DISCOVERY ##########
        (refs_string, guardados_id) = ref_discovery::ref_discovery(&r_path)?;
        let _ = stream.write(refs_string.as_bytes())?;
        // ########## ELECCION DE COMANDO ##########
        match elems[0] {
            "git-upload-pack" => {gitr_upload_pack(&mut stream, guardados_id, r_path)?;}, // Mandar al cliente
            "git-receive-pack" => {gitr_receive_pack(&mut stream, r_path)?;}, // Recibir del Cliente
            _ => {let _ = stream.write("Error: comando git no reconocido".as_bytes())?;}
        }
    }
    Ok(())
}

fn gitr_upload_pack(stream: &mut TcpStream, guardados_id: HashSet<String>, r_path: String) -> std::io::Result<()>{

    // ##########  PACKFILE NEGOTIATION ##########
    let (wants_id, haves_id) = packfile_negotiation(stream, guardados_id)?;
    // ########## PACKFILE DATA ##########
    if !wants_id.is_empty() {
        snd_packfile(stream, wants_id,haves_id, r_path)?;
    }
    
    Ok(())
}

fn gitr_receive_pack(stream: &mut TcpStream, r_path: String) -> std::io::Result<()> {
    // ##########  REFERENCE UPDATE ##########
    let mut buffer = [0;1024];
    
    if let Ok(n) = stream.read(&mut buffer) {
        let (old,new, names ) = get_changes(&buffer[..n])?;
        if old.is_empty() { //el cliente esta al dia
            return Ok(());
        }
        let pkt_needed = update_refs(old, new, names, r_path.clone())?;
        // ########## *PACKFILE DATA ##########
        if pkt_needed {
            let (ids, content) = rcv_packfile_bruno(stream)?;
            update_contents(ids, content, r_path.clone())?;
        } 
   
        return Ok(())
    }
    Err(Error::new(std::io::ErrorKind::Other, "Error: no se pudo leer el stream"))
}

fn update_contents(ids: Vec<String>, content: Vec<Vec<u8>>, r_path: String) -> std::io::Result<()> {
    if ids.len() != content.len() {
        return Err(Error::new(std::io::ErrorKind::Other, "Error: no coinciden los ids con los contenidos"))
    }
    let mut i = 0;
    for id in ids {
        let dir_path = format!("{}/objects/{}",r_path.clone(),id.split_at(2).0);
        let _ = fs::create_dir(dir_path.clone()); 
        let mut archivo = File::create(&format!("{}/{}",dir_path,id.split_at(2).1))?;
        archivo.write_all(&content[i])?;
        i += 1;
    }
    Ok(())
}

fn snd_packfile(stream: &mut TcpStream, wants_id: Vec<String>,haves_id: Vec<String>, r_path: String) -> std::io::Result<()> {
    let mut contents: Vec<Vec<u8>> = vec![];
    let all_commits = Commit::get_parents(wants_id.clone(), haves_id.clone(), r_path.clone()).unwrap_or(wants_id);
    let wants_id = Commit::get_objects_from_commits(all_commits.clone(), haves_id, r_path.clone()).unwrap_or(vec![]);
    for id in wants_id.clone() {
        match file_manager::get_object_bytes(id, r_path.clone()){
            Ok(obj) => contents.push(obj),
            Err(_) => return Err(Error::new(std::io::ErrorKind::InvalidInput, "Error: no se pudo obtener el objeto"))
        }
    }
    if let Ok(pack) = pack_data_bruno(contents) {
        let _ = stream.write(&pack)?;
    } else {
        return Err(Error::new(std::io::ErrorKind::InvalidInput, "Algo salio mal\n"))
    }
    Ok(())
}

fn packfile_negotiation(stream: &mut TcpStream, guardados_id: HashSet<String>) -> std::io::Result<(Vec<String>, Vec<String>)> {
    let (mut buffer, mut reply) = ([0; 1024], "0008NAK\n".to_string());
    let (mut wants_id, mut haves_id): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());    

    let mut n = stream.read(&mut buffer)?;
    let mut buf = Vec::from(&buffer[..n]);
    while n == 1024 {
        buffer = [0; 1024];
        n = stream.read(&mut buffer)?;
        buf.append(&mut Vec::from(&buffer[..n]));
    }
    let pkt_line = from_utf8(&buf).unwrap_or("");
    if pkt_line == "0000" { 
        return Ok((wants_id, haves_id));
    } 
    (wants_id, haves_id) = wants_n_haves(pkt_line.to_string(),wants_id,haves_id)?;
    
    for want in wants_id.clone() {
        if !guardados_id.contains(&want) {
            return  Err(Error::new(std::io::ErrorKind::InvalidInput, format!("Error: not our ref: {}\n",want)));

        }
    }
    for have in haves_id.clone() {
        if guardados_id.contains(&have) && reply == *"0008NAK\n" {
            reply = format!("003aACK {}\n", have.clone());
            let _ = stream.write(reply.as_bytes())?;
            break
        }
    }  
    if reply == *"0008NAK\n" {
        let _ = stream.write(reply.as_bytes())?;
    }
    Ok((wants_id, haves_id))
}

fn rcv_packfile_bruno(stream: &mut TcpStream) -> std::io::Result<(Vec<String>, Vec<Vec<u8>>)> {
    let mut buffer = Vec::new();
    let _ = stream.read_to_end(&mut buffer)?;
    let pack_file_struct = PackFile::new_from_server_packfile(&mut buffer);
    let pk_file = match pack_file_struct {
        Ok(pack_file) => {pack_file},
        _ => {return Err(Error::new(std::io::ErrorKind::InvalidInput, "Error: no se pudo crear el packfile"))}
    };
    let mut hashes: Vec<String> = Vec::new();
    let mut contents: Vec<Vec<u8>> = Vec::new();
    for object in pk_file.objects.iter(){
        match object{
            GitObject::Blob(blob) => {
                hashes.push(blob.get_hash());
                contents.push(blob.get_data());},
            GitObject::Commit(commit) => {
                hashes.push(commit.get_hash());
                contents.push(commit.get_data());},
            GitObject::Tree(tree) => {
                hashes.push(tree.get_hash());
                contents.push(tree.get_data());
            },
        }
    }
    Ok((hashes,contents))
}

fn update_refs(old: Vec<String>,new: Vec<String>, names: Vec<String>, r_path: String) -> std::io::Result<bool> {
    let nul_obj = "0000000000000000000000000000000000000000";
    let mut pkt_needed = false;
    for i in 0..old.len() {
        let path = r_path.clone() + "/" + &names[i];
        if old[i] == nul_obj  && new[i] != nul_obj{ 
            let mut new_file = File::create(&path)?;
            new_file.write_all(new[i].as_bytes())?;
            pkt_needed = true;
            continue
        } else if new[i] == nul_obj && old[i] != nul_obj { 
            fs::remove_file(&path)?;
            continue
        } else if old[i] == new[i] { 
            return Err(Error::new(std::io::ErrorKind::Other, "Error: el archivo no cambio")); // no se si es el error correcto
        } else { 
            pkt_needed = true;
            let path = path.replace('\\', "/");
            let old_file = fs::File::open(&path)?;
            let mut old_ref = String::new();
            BufReader::new(old_file).read_line(&mut old_ref)?;
            if old_ref == old[i] {
                let mut new_file = File::create(&path)?;
                new_file.write_all(new[i].as_bytes())?;
            } else {
                return Err(Error::new(std::io::ErrorKind::Other, "Error: nombre de archivo incorrecto"))
            }
        }
    }
    Ok(pkt_needed)
}

fn get_changes(buffer: &[u8]) -> std::io::Result<(Vec<String>,Vec<String>, Vec<String>)> {
    
    let changes = String::from_utf8_lossy(buffer);//.unwrap_or("Error");
    let mut old: Vec<String> = vec![];
    let mut new: Vec<String> = vec![];
    let mut names: Vec<String> = vec![];
    for change in changes.lines() {
        is_valid_pkt_line(&format!("{}\n",change))?;
        if change == "0000" {
            break
        }
        let elems: Vec<&str> = change.split_at(4).1.split(' ').collect(); // [old, new, ref-name]
        if elems.len() != 3 {
            return Err(Error::new(std::io::ErrorKind::Other, "Error: Negociacion Fallida"))
        }
        old.push(elems[0].to_string());
        new.push(elems[1].to_string());
        names.push(elems[2].to_string());
    }

    Ok((old, new, names))
}

fn pack_data_bruno(contents: Vec<Vec<u8>>) -> std::io::Result<Vec<u8>> {
    match create_packfile(prepare_contents(contents)) {
        Ok(pack) => Ok(pack),
        Err(_) => Err(Error::new(std::io::ErrorKind::Other, "Error: Armado de PACK fallido"))
    }
    
}

fn wants_n_haves(requests: String, mut wants: Vec<String>, mut haves: Vec<String>) -> std::io::Result<(Vec<String>,Vec<String>)> {
    let mut nuls_cont = 0;
    for line in requests.lines() {
        is_valid_pkt_line(&(line.to_string()+"\n"))?;
        let elems: Vec<&str> = line.split_at(4).1.split(' ').collect(); // [want/have, obj-id]
        if nuls_cont == 0 {
            match elems[0] {
                "want" => {wants.push(elems[1].to_string())},
                "" => {nuls_cont += 1;},// 0000
                "0009done" => {break},
                "0032have" => {
                    haves.push(elems[1].to_string());
                    nuls_cont += 1}
                _ => return Err(Error::new(std::io::ErrorKind::Other, "Error: Negociacion Fallida"))
            }
        } else if nuls_cont == 1 {
            match elems[0] {
                "have" => {haves.push(elems[1].to_string())},
                "" => {nuls_cont += 1}, // 0000
                "done" | "0009done" => {break},
                _ => return Err(Error::new(std::io::ErrorKind::Other, "Error: Negociacion Fallida"))
            }
        } else if nuls_cont == 2 {
            break
        }
    }
    Ok((wants,haves))
}

fn _capacidades() -> String {
    "capacidades-del-server ok_ok ...".to_string()
}

fn is_valid_pkt_line(pkt_line: &str) -> std::io::Result<()> {
    if !pkt_line.is_empty() && pkt_line.len() >= 4 && (usize::from_str_radix(pkt_line.split_at(4).0,16) == Ok(pkt_line.len()) || pkt_line.starts_with("0000")) {
        return Ok(())
    }
    Err(Error::new(std::io::ErrorKind::ConnectionRefused, "Error: No se sigue el estandar de PKT-LINE"))
}

/// # Devuelve
/// Una lista con los elementos de la linea de pkt-line: (comando, repo_local, repo_remoto)
fn split_n_validate_elems(pkt_line: &str) -> std::io::Result<Vec<&str>> {
    let line = pkt_line.split_at(4).1;
    let div1: Vec<&str> = line.split(' ').collect();
    if div1.len() < 2 {
        return Err(Error::new(std::io::ErrorKind::ConnectionRefused, "Error: No se sigue el estandar de PKT-LINE"))
    }

    let div2: Vec<&str> = div1[1].split('\0').collect(); 
    let mut elems: Vec<&str> = vec![];
    if (div1.len() == 2) || div2.len() == 3 {
        elems.push(div1[0]);
        elems.push(div2[0]);
        elems.push(div2[1].strip_prefix("host=").unwrap_or(div2[1]));
        return Ok(elems)

    }
    
    Err(Error::new(std::io::ErrorKind::ConnectionRefused, "Error: No se sigue el estandar de PKT-LINE"))
}

fn create_dirs(r_path: &str) -> std::io::Result<()> {
    let p_str = r_path.to_string();
    fs::create_dir(p_str.clone())?;
    write_file(p_str.clone() + "/HEAD", "ref: refs/heads/master".to_string())?;
    fs::create_dir(p_str.clone() + "/refs")?;
    fs::create_dir(p_str.clone() +"/refs/heads")?;
    fs::create_dir(p_str.clone() +"/refs/tags")?;
    fs::create_dir(p_str.clone() +"/objects")?;
    Ok(())
}

fn write_file(path: String, text: String) -> std::io::Result<()> {
    let mut archivo = File::create(path)?;
    archivo.write_all(text.as_bytes())?;
    Ok(())
}

