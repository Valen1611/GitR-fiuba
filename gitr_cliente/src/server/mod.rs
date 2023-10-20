extern crate flate2;
use std::collections::HashSet;
use std::fmt::format;
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
    let mut paso = 1;
    let mut guardados_id: HashSet<String> = HashSet::new();
    let mut refs_string :String;

    while let Ok(n) = stream.read(&mut buffer) {
        if n == 0 {
            // La conexión se cerró
            return Ok(());
        }

        if let Ok(request) = decode(&buffer) { 

            if paso == 1 {

                // ########## HANDSHAKE ##########
                let pkt_line = from_utf8(&request).unwrap_or(""); 
                match is_valid_pkt_line(pkt_line) {
                    Ok(_) => {},
                    Err(_) => {let _ = stream.write(&code("Error: no se respeta el formato pkt-line".as_bytes()).unwrap_or(vec![]));
                                return Ok(())}
                }
                let _elems = split_n_validate_elems(pkt_line)?;
                
                // ########## REFERENCE DISCOVERY ##########
                (refs_string, guardados_id) = ref_discovery(&r_path)?;
                if let Ok(reply) = code(refs_string.as_bytes()){
                    stream.write(&reply)?;
                    paso += 1;
                    continue;
                }
                return Err(Error::new(std::io::ErrorKind::InvalidInput, "Algo salio mal con el reference discovery"))

            } else if paso == 2 {

                // ##########  PACKFILE NEGOTIATION ##########
                let pkt_line = from_utf8(&request).unwrap_or("");
                if pkt_line == "0000" { // Cliente esta al dia
                    return Ok(())
                }
                if pkt_line == "0009done\n" {
                    paso +=1;
                    continue;
                }
                let (wants_id, haves_id) = wants_n_haves("cadena que envio el cliente".to_string())?;
                let mut reply = "0008NAK\n".to_string();
                for want in wants_id.clone() {
                    if !guardados_id.contains(&want) {
                        reply = "Error: wanted object not recognized\n".to_string()
                    }
                }
                for have in haves_id {
                    if guardados_id.contains(&have) && reply == "0008NAK\n".to_string() {
                        reply = format!("003aACK {}\n", have.clone());
                        break
                    }
                }
                if let Ok(reply) = code(reply.as_bytes()){
                    stream.write(&reply)?;
                    
                } else {
                    return Err(Error::new(std::io::ErrorKind::InvalidInput, "Algo salio mal\n"))
                }
                
                // ########## PACKFILE DATA ##########
                let pack = pack_data(wants_id, &r_path);

                

            }
                

        }
        println!("caca");

    }
    Err(Error::new(std::io::ErrorKind::Other, "Error en la conexion"))
}

fn pack_data(wants: Vec<String>, r_path: &String) -> std::io::Result<String> {
    if wants.len() > 9999 {
        return Err(Error::new(std::io::ErrorKind::Other, "Error: paquete demasiado grande"))
    }
    let mut txt = format!("PACK 0002 {}\n",wants.len());
    // ahora van todos los objetos asi: 
    // -  n-byte type and length (3-bit type, (n-1)*7+4-bit length)
    // -  compressed data

    // 1. 3-bit type: 1 = OBJ_COMMIT, 2 = OBJ_TREE, 3 = OBJ_BLOB, 4 = OBJ_TAG
    // 2. (n-1)*7+4-bit length: length of compressed data
    // 3. compressed data: zlib-compressed content of the object


    Ok(format!("ToDo"))
}

fn wants_n_haves(requests: String) -> std::io::Result<(Vec<String>,Vec<String>)> {
    let mut wants:Vec<String> = Vec::new();
    let mut haves: Vec<String> = Vec::new();
    let mut nuls_cont = 0;

    for line in requests.lines() {
        is_valid_pkt_line(&(line.to_string()+"\n"))?;
        let elems: Vec<&str> = line.split_at(4).1.split(" ").collect(); // [want/have, obj-id]
        if nuls_cont == 0 {
            match elems[0] {
                "want" => {wants.push(elems[1].to_string())},
                "" => {nuls_cont += 1;},   
                _ => return Err(Error::new(std::io::ErrorKind::Other, "Error: Negociacion Fallida"))
            }
        } else if nuls_cont == 1 {
            match elems[0] {
                "have" => {haves.push(elems[1].to_string())},
                "0000" => {nuls_cont += 1;},
                "done" => {break},
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
    ref_discovery_rec(r_path, &mut contenido_total,&mut guardados)?;
    contenido_total.push_str("0000\n");
    Ok((contenido_total,guardados))
}

fn ref_discovery_rec(dir_path: &str,contenido_total: &mut String, guardados: &mut HashSet<String>) -> std::io::Result<()> {
    for elem in fs::read_dir(dir_path)? {
        let elem = elem?;
        let ruta = elem.path();
        if ruta.is_file() {
            let mut contenido = String::new();
            let archivo = fs::File::open(&ruta)?;
            
            BufReader::new(archivo).read_line(&mut contenido)?;
            guardados.insert(contenido.clone());
            let path_str = ruta.to_str().unwrap_or("ERROR").strip_prefix(&format!("{}\\",dir_path)).unwrap_or("ERROR");
            let longitud = contenido.len() + path_str.len() + 6;
            let longitud_hex = format!("{:04x}", longitud);
            contenido_total.push_str(&longitud_hex);
            contenido_total.push_str(&contenido);
            contenido_total.push_str(&(" ".to_string() + path_str));
            // if let Some("HEAD") = elem.file_name().to_str() {
            //     contenido_total.push_str(&capacidades())
            // }
            contenido_total.push('\n');

        } else if ruta.is_dir() {
            ref_discovery_rec(ruta.to_str().unwrap_or(""), contenido_total, guardados)?;
        }
    }
    Ok(())
}

fn capacidades() -> String {
    "capacidades-del-server ok_ok ...".to_string()
}

fn is_valid_pkt_line(pkt_line: &str) -> std::io::Result<()> {
    if pkt_line != "" && pkt_line.len() >= 4 && (usize::from_str_radix(pkt_line.split_at(4).0,16) == Ok(pkt_line.len()) || pkt_line == "0000\n" ) {
        return Ok(())
    }
    Err(Error::new(std::io::ErrorKind::ConnectionRefused, "Error: No se sigue el estandar de PKT-LINE"))
}

fn split_n_validate_elems(pkt_line: &str) -> std::io::Result<Vec<&str>> {
    // 0033git-upload-pack /project.githost=myserver.com\0
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

fn main(){
      
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn inicializo_el_server_correctamente(){
        let address =  "127.0.0.1:5454";
        let builder_s = thread::Builder::new().name("server".to_string());
        let builder_c = thread::Builder::new().name("cliente".to_string());

        let server = builder_s.spawn(||{
            server_init("remote_repo", address).unwrap();
        }).unwrap();
        
        let client = builder_c.spawn(move||{
            let mut socket = TcpStream::connect(address).unwrap();
            socket.write(&code("Hola server".as_bytes()).unwrap());
            let mut buffer = [0; 1024];
            
            socket.read(&mut buffer).unwrap();
            println!("llego antes del assert");
            println!("[[[[[[{:?}]]]]]]]]",from_utf8(&decode(&buffer).unwrap()).unwrap());
            assert_eq!(from_utf8(&decode(&buffer).unwrap()), Ok("Error: no se respeta el formato pkt-line"));
            print!("ok");
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
        let (wants,haves) = wants_n_haves(input.to_string()).unwrap();
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
        assert_eq!(refs_string, "00327217a7c7e582c46cec22a130adf4b9d7d950fba0 HEAD\n0000\n")
    }
}
