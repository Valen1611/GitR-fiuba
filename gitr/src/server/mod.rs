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
use flate2::read::ZlibDecoder;

use crate::git_transport::pack_file::PackFile;
use crate::objects::commit::Commit;
use crate::objects::git_object::GitObject;
use crate::objects::tree::Tree;


pub fn server_init (r_path: &str, s_addr: &str) -> std::io::Result<()>  {
    let _ = create_dirs(&r_path);
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
            // La conexión se cerró
            return Ok(());
        }
        
        // ########## HANDSHAKE ##########
        let pkt_line = from_utf8(&buffer).unwrap_or(""); 
        match is_valid_pkt_line(pkt_line) {
            Ok(_) => {},
            Err(_) => {let _ = stream.write(&"Error: no se respeta el formato pkt-line".as_bytes());
            return Ok(())}
        }
        let elems = split_n_validate_elems(pkt_line)?;
        
        // ########## REFERENCE DISCOVERY ##########
        
        (refs_string, guardados_id) = ref_discovery(&r_path)?;
        stream.write(&refs_string.as_bytes())?;

        // ########## ELECCION DE COMANDO ##########
        match elems[0] {
            "gitr-upload-pack" => {gitr_upload_pack(&mut stream, guardados_id, r_path)?;}, // Mandar al cliente
            "gitr-receive-pack" => {gitr_receive_pack(&mut stream, r_path)?;}, // Recibir del Cliente
            _ => {stream.write(&"Error: comando git no reconocido".as_bytes())?;}
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
    stream.read(&mut buffer)?;
    let (old,new, names ) = get_changes(&buffer)?;
    let pkt_needed = update_refs(old, new, names, r_path.clone())?;

    // ########## *PACKFILE DATA ##########
    if pkt_needed {
        let (ids, content) = rcv_packfile_bruno(stream)?;
        update_contents(ids, content, r_path.clone())?;
    } 
   
    Ok(())
}

/// INCOMPLETA -> ERA DEL CLIENTE
fn _make_push_cert(ids: Vec<String>, r_path: String, host: String) -> std::io::Result<String> {
    let mut push_cert = String::new();
    let mut line: &str = "push-cert0\n";
    push_cert.push_str(&format!("{:04x}{}",line.len(), line));
    line = "certificate version 0.1\n";
    push_cert.push_str(&format!("{:04x}{}",line.len(), line));
    let mut ident: Vec<&str> = vec!["nombre","mail"];
    let mut obj: String;
    for id in ids {
        obj = get_object(id, r_path.clone())?;
        if _is_commit(obj.clone()) {
            let div1: Vec<&str> = obj.split("committer ").collect();
            let div2: Vec<&str> = div1[1].split(">").collect();
            ident = div2[0].split(" <").collect();
            break
        }
    }
    let line: &str = &format!("pusher {} {}\n",ident[0],ident[1]);
    push_cert.push_str(&format!("{:04x}{}",line.len(), line));
    let line: &str = &format!("pushee {}\n",host);
    push_cert.push_str(&format!("{:04x}{}",line.len(), line));
    // ...
    // ... ...
    // ... ... ...
    Ok("".to_string())
}

fn _is_commit(obj: String) -> bool {
    let mut lines = obj.lines();
    let first_line = lines.next().unwrap_or("");
    if first_line == "tree" {
        return true
    }
    false
}

fn get_object(id: String, r_path: String) -> std::io::Result<String> {
    let dir_path = format!("{}/objects/{}",r_path.clone(),id.split_at(2).0);
    let mut archivo = File::open(&format!("{}/{}",dir_path,id.split_at(2).1))?; // si no existe tira error
    let mut contenido: Vec<u8>= Vec::new();
    archivo.read_to_end(&mut contenido)?;
    let descomprimido = String::from_utf8_lossy(&decode(&contenido)?).to_string();
    Ok(descomprimido)
}

fn update_contents(ids: Vec<String>, content: Vec<Vec<u8>>, r_path: String) -> std::io::Result<()> {
    if ids.len() != content.len() {
        return Err(Error::new(std::io::ErrorKind::Other, "Error: no coinciden los ids con los contenidos"))
    }
    let mut i = 0;
    for id in ids {
        
        let dir_path = format!("{}/objects/{}",r_path.clone(),id.split_at(2).0);
        let _ = fs::create_dir(dir_path.clone()); // si ya existe tira error pero no pasa nada
        let mut archivo = File::create(&format!("{}/{}",dir_path,id.split_at(2).1))?;
        archivo.write_all(&code(&content[i])?)?;
        i += 1;
    }
    Ok(())
}

fn snd_packfile(stream: &mut TcpStream, wants_id: Vec<String>,haves_id: Vec<String>, r_path: String) -> std::io::Result<()> {
    let mut contents: Vec<String> = vec![];
    let wants_id = get_objects_from_commits(wants_id.clone(), haves_id, r_path.clone())?;
    for id in wants_id.clone() {
        contents.push(get_object(id, r_path.clone())?);
    }

    if let Ok(pack_string) = pack_data_bruno(wants_id, contents) {
        stream.write(&pack_string.as_bytes())?;
    } else {
        return Err(Error::new(std::io::ErrorKind::InvalidInput, "Algo salio mal\n"))
    }
    Ok(())
}

fn get_objects_from_commits(commits_id: Vec<String>,client_objects: Vec<String>, r_path: String) -> std::io::Result<Vec<String>> {
    // Voy metiendo en el objects todo lo que no haya que mandarle denuevo al cliente
    let mut object_ids: HashSet<String> = HashSet::new();
    for obj_id in client_objects.clone() {
        object_ids.insert(obj_id);
    }
    let mut commits: Vec<Commit> = Vec::new();
    for id in commits_id {
        match Commit::new_commit_from_string(get_object(id, r_path.clone())?) {
            Ok(commit) => {commits.push(commit)},
            _ => {return Err(Error::new(std::io::ErrorKind::InvalidInput, "Error: no se pudo crear el commit"))}
        }
    } // Ahora tengo los Commits como objeto en el vector commits
    for commit in commits {
        object_ids.insert(commit.get_tree());
        get_tree_objects(commit.get_tree(), r_path.clone(), &mut object_ids)?;
    }
    // Sacamos los que ya tiene el cliente
    for obj in client_objects{
        object_ids.remove(&obj);
    } 
    Ok(Vec::from_iter(object_ids.into_iter()))

    
}

fn get_tree_objects(tree_id: String, r_path: String, object_ids: &mut HashSet<String>) -> std::io::Result<()> {
    // tree <content length><NUL><file mode> <filename><NUL><item sha><file mode> <filename><NUL><item sha><file mode> <filename><NUL><item sha>...

    let tree_objects = match Tree::get_objects_id_from_string(get_object(tree_id, r_path.clone())?){
        Ok(ids) => {ids},
        _ => {return Err(Error::new(std::io::ErrorKind::InvalidInput, "Error: no se pudo crear el arbol"))}
    };
    for obj_id in tree_objects {
        object_ids.insert(obj_id.clone());
        let _ = get_tree_objects(obj_id.clone(), r_path.clone(),object_ids); 
    }
    Ok(())
}

fn packfile_negotiation(stream: &mut TcpStream, guardados_id: HashSet<String>) -> std::io::Result<(Vec<String>, Vec<String>)> {
    let (mut buffer, mut reply) = ([0; 1024], "0008NAK\n".to_string());
    let (mut wants_id, mut haves_id): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());    
    loop {
        stream.read(&mut buffer)?;
        let pkt_line = from_utf8(&buffer).unwrap_or("");
        if pkt_line == "0000" { // si la primera linea es 0000, el cliente esta al dia y no hay mas que hacer con el
            return Ok((wants_id, haves_id));
        } else if pkt_line == "0009done\n" { // el cliente cierra el packet-file
            break;
        }
        (wants_id, haves_id) = wants_n_haves(String::from_utf8_lossy(&buffer).to_string(),wants_id,haves_id)?;
        for want in wants_id.clone() {
            if !guardados_id.contains(&want) {
                return  Err(Error::new(std::io::ErrorKind::InvalidInput, format!("Error: not our ref: {}\n",want)));

            }
        }
        for have in haves_id.clone() {
            if guardados_id.contains(&have) && reply == "0008NAK\n".to_string() {
                reply = format!("003aACK {}\n", have.clone());
                stream.write(&reply.as_bytes())?;
                break
            }
        }  
    }
    if reply == "0008NAK\n".to_string() {
        stream.write(&reply.as_bytes())?;
    }
    Ok((wants_id, haves_id))
}

fn rcv_packfile_bruno(stream: &mut TcpStream) -> std::io::Result<(Vec<String>, Vec<Vec<u8>>)> {
    let mut buffer: [u8;1024] = [0; 1024];
    stream.read(&mut buffer)?;
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
        if old[i] == nul_obj  && new[i] != nul_obj{ // caso de creacion de archivo
            let mut new_file = File::create(&path)?;
            new_file.write_all(new[i].as_bytes())?;
            pkt_needed = true;
            continue
        } else if new[i] == nul_obj && old[i] != nul_obj { // caso de borrado de archivo
            fs::remove_file(&path)?;
            continue
        } else if old[i] == new[i] { // caso de archivo sin cambios
            return Err(Error::new(std::io::ErrorKind::Other, "Error: el archivo no cambio")); // no se si es el error correcto
        } else { // caso de archivo modificado
            pkt_needed = true;
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
    let changes = from_utf8(&buffer).unwrap_or("");
    let mut old: Vec<String> = vec![];
    let mut new: Vec<String> = vec![];
    let mut names: Vec<String> = vec![];
    for change in changes.lines() {
        is_valid_pkt_line(&format!("{}\n",change))?;
        if change == "0000" {
            break
        }
        let elems: Vec<&str> = change.split_at(4).1.split(" ").collect(); // [old, new, ref-name]
        if elems.len() != 3 {
            return Err(Error::new(std::io::ErrorKind::Other, "Error: Negociacion Fallida"))
        }
        old.push(elems[0].to_string());
        new.push(elems[1].to_string());
        names.push(elems[2].to_string());
    }

    Ok((old, new, names))
}

fn pack_data_bruno(_ids: Vec<String>, _contents: Vec<String>) -> std::io::Result<String> {
    
    Ok(format!("ToDo"))
}

fn wants_n_haves(requests: String, mut wants: Vec<String>, mut haves: Vec<String>) -> std::io::Result<(Vec<String>,Vec<String>)> {
    
    let mut nuls_cont = 0;

    for line in requests.lines() {
        is_valid_pkt_line(&(line.to_string()+"\n"))?;
        
        let elems: Vec<&str> = line.split_at(4).1.split(" ").collect(); // [want/have, obj-id]
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
    let mut contenido = String::new();
    let archivo = fs::File::open(&ruta)?;

    BufReader::new(archivo).read_line(&mut contenido)?;
    guardados.insert(contenido.clone());
    let longitud = contenido.len() + 10;
    let longitud_hex = format!("{:04x}", longitud);
    contenido_total.push_str(&longitud_hex);
    contenido_total.push_str(&contenido);
    contenido_total.push_str(&(" ".to_string() + "HEAD"));
    contenido_total.push('\n');
    
    let refs_path = format!("{}/refs",r_path);
    ref_discovery_dir(&refs_path,r_path, &mut contenido_total,&mut guardados)?;
    
    contenido_total.push_str("0000\n");
    
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
            let path_str = ruta.to_str().unwrap_or("ERROR").strip_prefix(&format!("{}\\",original_path)).unwrap_or("ERROR");
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
    println!("ok {:?}\n",pkt_line);
    if pkt_line != "" && pkt_line.len() >= 4 && (usize::from_str_radix(pkt_line.split_at(4).0,16) == Ok(pkt_line.len()) || pkt_line == "0000\n" || pkt_line == "0000" || pkt_line == "0009done\n" || pkt_line == "00000009done" ) {
        return Ok(())
    }
    Err(Error::new(std::io::ErrorKind::ConnectionRefused, "Error: No se sigue el estandar de PKT-LINE"))
}
/// Devuelve un vector con los elementos de la linea de pkt_line { orden, repo_local_path, host }
fn split_n_validate_elems(pkt_line: &str) -> std::io::Result<Vec<&str>> {
    // 0033git-upload-pack /project.git\0host=myserver.com\0
    let line = pkt_line.split_at(4).1;
    let div1: Vec<&str> = line.split(" ").collect(); // [comando , resto] 
    if div1.len() < 2 {
        return Err(Error::new(std::io::ErrorKind::ConnectionRefused, "Error: No se sigue el estandar de PKT-LINE"))
    }

    let div2: Vec<&str> = div1[1].split("\0").collect(); // [/repo_local_path, "0" + gitr.com???, "0"]
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
    write_file(p_str.clone() + "/HEAD", "7217a7c7e582c46cec22a130adf4b9d7d950fba0".to_string())?;
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

fn decode(input: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut decoder = ZlibDecoder::new(input);
    let mut decoded_data = Vec::new();
    decoder.read_to_end(&mut decoded_data)?;
    Ok(decoded_data)
}

fn _main(){
      
}

#[cfg(test)]
mod tests{

    use super::*;

    #[test]
    #[ignore = "Hay que frenarlo manualmente"]
    fn inicializo_el_server_correctamente(){
        let address =  "127.0.0.1:5454";
        let builder_s = thread::Builder::new().name("server".to_string());
        let builder_c = thread::Builder::new().name("cliente".to_string());

        let server = builder_s.spawn(||{
            server_init("remote_repo", address).unwrap();
        }).unwrap();
        
        let client = builder_c.spawn(move||{
            let mut socket = TcpStream::connect(address).unwrap();
            socket.write(&"Hola server".as_bytes()).unwrap();
            let mut buffer = [0; 1024];
            
            socket.read(&mut buffer).unwrap();
            assert_eq!(from_utf8(&decode(&buffer).unwrap()), Ok("Error: no se respeta el formato pkt-line"));
            return ;
        }).unwrap();

        server.join().unwrap();
        client.join().unwrap();
    }

    #[test]
    fn test01_encoder_decoder() {
        let input = "Hola mundo".as_bytes();
        let encoded = code(input).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert_eq!(input, decoded.as_slice());
    }

    #[test]
    fn test02_split_n_validate(){
        let pkt_line = "0033git-upload-pack /project.git\0host=myserver.com\0".to_string();
        let elems = split_n_validate_elems(&pkt_line).unwrap();
        assert_eq!(elems[0], "git-upload-pack");
        assert_eq!(elems[1], "/project.git");
        assert_eq!(elems[2], "host=myserver.com");
    }

    #[test]
    fn test03_is_valid_pkt_line(){
        is_valid_pkt_line("")
            .expect_err("Error: No se sigue el estandar de PKT-LINE");
        is_valid_pkt_line("132")
            .expect_err("Error: No se sigue el estandar de PKT-LINE");
        is_valid_pkt_line("0000hola")
            .expect_err("Error: No se sigue el estandar de PKT-LINE");
        is_valid_pkt_line("kkkkhola")
            .expect_err("Error: No se sigue el estandar de PKT-LINE");
        assert!(is_valid_pkt_line("0000").is_ok());
        assert!(is_valid_pkt_line("000ahola:)").is_ok());
        assert!(is_valid_pkt_line("0033git-upload-pack /project.git\0host=myserver.com\0").is_ok());
    }

    #[test]
    fn test04_wants_n_haves(){
        let input = {
            "0032want 74730d410fcb6603ace96f1dc55ea6196122532d
0032want 7d1665144a3a975c05f1f43902ddaf084e784dbe
0032want 5a3f6be755bbb7deae50065988cbfa1ffa9ab68a
0032want 7e47fe2bd8d01d481f44d7af0531bd93d3b21c01
0032want 74730d410fcb6603ace96f1dc55ea6196122532d
0000
0009done"};
        let (wants,haves) = wants_n_haves(input.to_string(),Vec::new(), Vec::new()).unwrap();
        assert_eq!(wants[0], "74730d410fcb6603ace96f1dc55ea6196122532d");
        assert_eq!(wants[1], "7d1665144a3a975c05f1f43902ddaf084e784dbe");
        assert_eq!(wants[2], "5a3f6be755bbb7deae50065988cbfa1ffa9ab68a");
        assert_eq!(wants[3], "7e47fe2bd8d01d481f44d7af0531bd93d3b21c01");
        assert_eq!(wants[4], "74730d410fcb6603ace96f1dc55ea6196122532d");
        assert!(haves.is_empty());
    }

    #[test]
    fn test05_ref_discovery() {
        let (refs_string,_guardados) = ref_discovery("remote_repo").unwrap();
        print!("{}\n",refs_string);
        let ref_lines: Vec<&str> = refs_string.lines().collect();
        assert_eq!(ref_lines.first(), Some(&"00327217a7c7e582c46cec22a130adf4b9d7d950fba0 HEAD"));
        assert_eq!(ref_lines.last(),Some(&"0000"))
    }

    #[test]
    fn test06_get_changes() {
        let input = {
            "00677d1665144a3a975c05f1f43902ddaf084e784dbe 74730d410fcb6603ace96f1dc55ea6196122532d refs/heads/debug
006874730d410fcb6603ace96f1dc55ea6196122532d 5a3f6be755bbb7deae50065988cbfa1ffa9ab68a refs/heads/master
0000"};
        let (old,new,names) = get_changes(input.as_bytes()).unwrap();
        assert_eq!(old[0], "7d1665144a3a975c05f1f43902ddaf084e784dbe");
        assert_eq!(old[1], "74730d410fcb6603ace96f1dc55ea6196122532d");
        assert_eq!(new[0], "74730d410fcb6603ace96f1dc55ea6196122532d");
        assert_eq!(new[1], "5a3f6be755bbb7deae50065988cbfa1ffa9ab68a");
        assert_eq!(names[0], "refs/heads/debug");
        assert_eq!(names[1], "refs/heads/master");
    }

    #[test]
    fn test07_update_refs() {
        let r_path = "remote_repo";
        let _ = create_dirs(r_path);
        assert!(fs::metadata(format!("{}/refs/heads/debug",r_path)).is_err());
        assert!(fs::metadata(format!("{}/refs/heads/master",r_path)).is_err());
        // caso de creacion de archivo
        let old = vec!["0000000000000000000000000000000000000000".to_string(),"0000000000000000000000000000000000000000".to_string()];
        let new = vec!["74730d410fcb6603ace96f1dc55ea6196122532d".to_string(),"5a3f6be755bbb7deae50065988cbfa1ffa9ab68a".to_string()];
        let names = vec!["refs/heads/debug".to_string(),"refs/heads/master".to_string()];
        let pkt_needed = update_refs(old, new, names, r_path.to_string()).unwrap();
        assert!(pkt_needed);
        assert!(fs::metadata(format!("{}/refs/heads/debug",r_path)).is_ok());
        assert!(fs::metadata(format!("{}/refs/heads/master",r_path)).is_ok());
        assert_eq!(fs::read_to_string(format!("{}/refs/heads/debug",r_path)).unwrap_or("".to_string()), "74730d410fcb6603ace96f1dc55ea6196122532d");
        assert_eq!(fs::read_to_string(format!("{}/refs/heads/master",r_path)).unwrap_or("".to_string()), "5a3f6be755bbb7deae50065988cbfa1ffa9ab68a");
        
        // caso modificacion de archivo
        let old = vec!["74730d410fcb6603ace96f1dc55ea6196122532d".to_string(),"5a3f6be755bbb7deae50065988cbfa1ffa9ab68a".to_string()];
        let new = vec!["7d1665144a3a975c05f1f43902ddaf084e784dbe".to_string(),"74730d410fcb6603ace96f1dc55ea6196122532d".to_string()];
        let names = vec!["refs/heads/debug".to_string(),"refs/heads/master".to_string()];
        let pkt_needed = update_refs(old, new, names, r_path.to_string()).unwrap();
        assert!(pkt_needed);
        assert!(fs::metadata(format!("{}/refs/heads/debug",r_path)).is_ok());
        assert!(fs::metadata(format!("{}/refs/heads/master",r_path)).is_ok());
        assert_eq!(fs::read_to_string(format!("{}/refs/heads/debug",r_path)).unwrap_or("".to_string()), "7d1665144a3a975c05f1f43902ddaf084e784dbe");
        assert_eq!(fs::read_to_string(format!("{}/refs/heads/master",r_path)).unwrap_or("".to_string()), "74730d410fcb6603ace96f1dc55ea6196122532d");
        // caso de borrado de archivo
        let old = vec!["7d1665144a3a975c05f1f43902ddaf084e784dbe".to_string(),"74730d410fcb6603ace96f1dc55ea6196122532d".to_string()];
        let new = vec!["0000000000000000000000000000000000000000".to_string(),"0000000000000000000000000000000000000000".to_string()];
        let names = vec!["refs/heads/debug".to_string(),"refs/heads/master".to_string()];
        let pkt_needed = update_refs(old, new, names, r_path.to_string()).unwrap();
        assert!(!pkt_needed);
        assert!(fs::metadata(format!("{}/refs/heads/debug",r_path)).is_err());
        assert!(fs::metadata(format!("{}/refs/heads/master",r_path)).is_err());
    }

    #[test]
    fn test08_update_contents_n_get_object() {
        let r_path = "remote_repo";
        let _ = create_dirs(r_path);
        let ids = vec!["74730d410fcb6603ace96f1dc55ea6196122532d".to_string(),"5a3f6be755bbb7deae50065988cbfa1ffa9ab68a".to_string()];
        let content: Vec<Vec<u8>> = vec!["Hola mundo".to_string().as_bytes().to_vec(),"Chau mundo".to_string().as_bytes().to_vec()];
        update_contents(ids, content, r_path.to_string()).unwrap();
        assert_eq!(get_object("74730d410fcb6603ace96f1dc55ea6196122532d".to_string(), r_path.to_string()).unwrap(), "Hola mundo");
        assert_eq!(get_object("5a3f6be755bbb7deae50065988cbfa1ffa9ab68a".to_string(), r_path.to_string()).unwrap(), "Chau mundo");         

    }

}
