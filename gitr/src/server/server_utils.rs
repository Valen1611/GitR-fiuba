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
use flate2::Compression;
use flate2::write::ZlibEncoder;
use crate::file_manager;
use crate::git_transport::pack_file::PackFile;
use crate::git_transport::pack_file::create_packfile;
use crate::git_transport::pack_file::prepare_contents;

use crate::objects::commit::Commit;
use crate::objects::git_object::GitObject;


pub fn server_init (r_path: &str, s_addr: &str) -> std::io::Result<()>  {
    let _ = create_dirs(r_path);
    let listener = TcpListener::bind(s_addr)?;
    let mut childs = Vec::new();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {                
                let clon = r_path.to_string();
                let builder = thread::Builder::new().name("cliente_random".to_string());

                childs.push(builder.spawn(|| {handle_client(stream,clon)})?);
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

fn handle_client(mut stream: TcpStream, r_path: String) -> std::io::Result<()> {

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
        // ########## REFERENCE DISCOVERY ##########
        (refs_string, guardados_id) = ref_discovery(&r_path)?;
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
    let mut buffer = [0; 1024];
    
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

fn _is_commit(obj: String) -> bool {
    let mut lines = obj.lines();
    let first_line = lines.next().unwrap_or("");
    if first_line == "tree" {
        return true
    }
    false
}



fn update_contents(ids: Vec<String>, content: Vec<Vec<u8>>, r_path: String) -> std::io::Result<()> {
    if ids.len() != content.len() {
        return Err(Error::new(std::io::ErrorKind::Other, "Error: no coinciden los ids con los contenidos"))
    }
    println!("check1");
    let mut i = 0;
    for id in ids {
        println!("check2");
        let dir_path = format!("{}/objects/{}",r_path.clone(),id.split_at(2).0);
        let _ = fs::create_dir(dir_path.clone()); 
        let mut archivo = File::create(&format!("{}/{}",dir_path,id.split_at(2).1))?;
        println!("check3");
        archivo.write_all(&content[i])?;
        println!("check4");
        i += 1;
    }
    Ok(())
}

fn snd_packfile(stream: &mut TcpStream, wants_id: Vec<String>,haves_id: Vec<String>, r_path: String) -> std::io::Result<()> {
    let mut contents: Vec<Vec<u8>> = vec![];
    let wants_id = Commit::get_objects_from_commits(wants_id.clone(), haves_id, r_path.clone()).unwrap_or(vec![]);
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
    loop {
        let n = stream.read(&mut buffer)?;
        let pkt_line = from_utf8(&buffer[..n]).unwrap_or("");
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
        if pkt_line.ends_with("0009done\n") { 
            break;
        }
    }
    if reply == *"0008NAK\n" {
        let _ = stream.write(reply.as_bytes())?;
    }
    Ok((wants_id, haves_id))
}

fn rcv_packfile_bruno(stream: &mut TcpStream) -> std::io::Result<(Vec<String>, Vec<Vec<u8>>)> {
    let mut buffer: [u8;1024] = [0; 1024];
    let _ = stream.read(&mut buffer)?;
    let pack_file_struct = PackFile::new_from_server_packfile(&mut buffer);
    let pk_file = match pack_file_struct {
        Ok(pack_file) => {pack_file},
        _ => {return Err(Error::new(std::io::ErrorKind::InvalidInput, "Error: no se pudo crear el packfile"))}
    };
    let mut hashes: Vec<String> = Vec::new();
    let mut contents: Vec<Vec<u8>> = Vec::new();
    for object in pk_file.objects.iter(){
        println!("objeto: {object:?}\n");
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

    let changes = from_utf8(buffer).unwrap_or("");
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

fn ref_discovery(r_path: &str) -> std::io::Result<(String,HashSet<String>)> {
    let mut contenido_total = String::new();
    let mut guardados: HashSet<String> = HashSet::new();
    let ruta = format!("{}/HEAD",r_path);
    let mut cont = String::new();
    let archivo = fs::File::open(ruta)?;
    BufReader::new(archivo).read_line(&mut cont)?;
    
    let c =r_path.to_string() +"/"+ cont.split_at(5).1;
    
    let mut contenido = "".to_string();
    if let Ok(f) = fs::File::open(c){
        BufReader::new(f).read_line(&mut contenido)?;
        guardados.insert(contenido.clone());
        let longitud = contenido.len() + 10;
        let longitud_hex = format!("{:04x}", longitud);
        contenido_total.push_str(&longitud_hex);
        contenido_total.push_str(&contenido);
        contenido_total.push_str(&(" ".to_string() + "HEAD"));
        contenido_total.push('\n');
    }

    
    let refs_path = format!("{}/refs",r_path);
    ref_discovery_dir(&(refs_path.clone() + "/heads"),r_path, &mut contenido_total,&mut guardados)?;
    ref_discovery_dir(&(refs_path + "/tags"),r_path, &mut contenido_total,&mut guardados)?;
    
    contenido_total.push_str("0000");
    
    Ok((contenido_total,guardados))
}
    
fn ref_discovery_dir(dir_path: &str,original_path: &str,contenido_total: &mut String, guardados: &mut HashSet<String>) -> std::io::Result<()> {
    for elem in fs::read_dir(dir_path)? {
        let elem = elem?;
        let ruta = elem.path();
        if ruta.is_file() {
            let mut contenido = String::new();
            let archivo = fs::File::open(&ruta)?;
            BufReader::new(archivo).read_line(&mut contenido)?;
            guardados.insert(contenido.clone());
            let path_str = ruta.to_str().unwrap_or("ERROR").strip_prefix(&format!("{}/",original_path)).unwrap_or("ERROR2");
            let path_str = &path_str.replace('/', "\\");
            let longitud = contenido.len() + path_str.len() + 6;
            let longitud_hex = format!("{:04x}", longitud);
            contenido_total.push_str(&longitud_hex);
            contenido_total.push_str(&contenido);
            contenido_total.push_str(&(" ".to_string() + path_str));
            contenido_total.push('\n');

        } 
    }
    Ok(())
}

fn _capacidades() -> String {
    "capacidades-del-server ok_ok ...".to_string()
}

fn is_valid_pkt_line(pkt_line: &str) -> std::io::Result<()> {
    println!("{pkt_line:?}");
    if !pkt_line.is_empty() && pkt_line.len() >= 4 && (usize::from_str_radix(pkt_line.split_at(4).0,16) == Ok(pkt_line.len()) || pkt_line == "0000\n" || pkt_line == "0000" || pkt_line == "0009done\n" || pkt_line == "00000009done" || pkt_line == "00000009done\n" ) {
        return Ok(())
    }
    Err(Error::new(std::io::ErrorKind::ConnectionRefused, "Error: No se sigue el estandar de PKT-LINE"))
}
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
        elems.push(div2[1]);
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

fn code(input: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(input)?;
    encoder.finish()
}

