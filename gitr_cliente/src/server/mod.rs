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
    create_dirs(&r_path)?;
    println!("{}",s_addr);
    let listener = TcpListener::bind(s_addr)?;
    let mut childs = Vec::new();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("{:?}",stream);
                let clon = r_path.to_string();
                childs.push(thread::spawn(|| {handle_client(stream,clon)}));
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
                is_valid_pkt_line(pkt_line)?;
                let _elems = split_n_validate_elems(pkt_line)?;
                
                // ########## REFERENCE DISCOVERY ##########
                (refs_string, guardados_id) = ref_discovery(&r_path)?;
                if let Ok(reply) = code(refs_string.as_bytes()){
                    stream.write(&reply)?;
                    paso += 1;
                    continue;
                }
                return Err(Error::new(std::io::ErrorKind::InvalidInput, "Algo salio mal con"))

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
                let mut shared = "".to_string();
                for want in wants_id {
                    if !guardados_id.contains(&want) {
                        shared = "Error".to_string()
                    }
                }
                for have in haves_id {
                    if guardados_id.contains(&have){
                        shared = have.clone();
                        break
                    }
                }
                let reply = match shared.as_str() {
                    "" => "0008NAK\n".to_string(),
                    _ => format!("003aACK {}\n",shared)
                };

                

            }
                

        }
    }
    Err(Error::new(std::io::ErrorKind::Other, "Error en la conexion"))
}


fn server_handler(request: Vec<u8>, r_path: &str, stream: &TcpStream ) -> std::io::Result<Vec<u8>> {

    // 1) comando inicial y datos
    // ej: 003egit-upload-pack/project.git\0host=myserver.com\0
    let pkt_line = from_utf8(&request).unwrap_or(""); 
    is_valid_pkt_line(pkt_line)?;
    let _elems = split_n_validate_elems(pkt_line)?;
    //  por ahi juntar estas 3 lineas en una funcion

    // Server presenta lo que tiene
    let (_refs_string, guardados) = ref_discovery(r_path)?;

    // Negociacion: 
    // - Cliente manda [want-lines, NUL, *de a 32 have-lines terminados por NUL, done]
    let (wants, haves) = wants_n_haves("cadena que envio el cliente".to_string())?;

    // - Server responde [*Err (si hay un obj-id que no tiene), *ACK obj-id en el primero que compartan/ NAK si no comparten nada todavia]

    //    -- Despues del done el Server manda [ACK + (id del ultimo comit que comparten)/ NAK si no comparten nada]

    // [Envio de PACKFILE DATA]



    Ok(request)
}

fn wants_n_haves(requests: String) -> std::io::Result<(Vec<String>,Vec<String>)> {
    let mut wants:Vec<String> = Vec::new();
    let mut haves: Vec<String> = Vec::new();
    let mut nuls_cont = 0;

    for line in requests.lines() {
        is_valid_pkt_line(line)?;
        let elems: Vec<&str> = line.split_at(4).1.split(" ").collect(); // [want/have, obj-id]
        if nuls_cont == 0 {
            match elems[0] {
                "have" => {haves.push(elems[1].to_string())},
                "0000" => {nuls_cont += 1;},   
                _ => return Err(Error::new(std::io::ErrorKind::Other, "Error: Negociacion Fallida"))
            }
        } else if nuls_cont == 1 {
            match elems[0] {
                "want" => {wants.push(elems[1].to_string())},
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
    contenido_total.push_str("0000");
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
            contenido_total.push_str(&contenido);
            contenido_total.push_str(&(" ".to_string() + elem.path().to_str().unwrap_or("ERROR")));
            if let Some("HEAD") = elem.file_name().to_str() {
                contenido_total.push_str(&capacidades())
            }
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
    if pkt_line != "" && pkt_line.len() < 4 && (usize::from_str_radix(pkt_line.split_at(4).0,16) != Ok(pkt_line.len()) || pkt_line == "0000" ) {
        return Ok(())
    }
    Err(Error::new(std::io::ErrorKind::ConnectionRefused, "Error: No se sigue el estandar de PKT-LINE"))
}

fn split_n_validate_elems(pkt_line: &str) -> std::io::Result<Vec<&str>> {
    let line = pkt_line.split_at(4).1;
    let div1: Vec<&str> = line.split("/").collect(); // [comando , resto] 
    let div2: Vec<&str> = div1[1].split("\'").collect(); // [repo_local_path, "0" + gitr.com???, "0"]
    let mut elems: Vec<&str> = vec![];
    if (div1.len() == 2) || div2.len() == 3 {
        if let Some(host) = div2[1].strip_prefix("0") {
            elems.push(div1[0]);
            elems.push(div2[0]);
            elems.push(host);
            return Ok(elems)
        }
    }
    Err(Error::new(std::io::ErrorKind::ConnectionRefused, "Error: No se sigue el estandar de PKT-LINE"))
}

fn create_dirs(r_path: &str) -> std::io::Result<()> {
    let p_str = r_path.to_string();
    fs::create_dir(p_str.clone())?;
    write_file(p_str.clone() + "/HEAD", "ToDo".to_string())?;
    fs::create_dir(p_str.clone() + "/refs")?;
    fs::create_dir(p_str.clone() +"refs/heads")?;
    fs::create_dir(p_str.clone() +"refs/remotes")?;
    fs::create_dir(p_str.clone() +"refs/remotes/origin")?;
    fs::create_dir(p_str.clone() +"refs/tags")?;
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
    let handler = thread::spawn(||{
        server_init("remote_repo", "127.0.0.1:5454");
    });

    handler.join().unwrap();
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn inicializo_el_server_correctamente(){
        let address =  "127.0.0.1:5454";
        let server = thread::spawn(||{
            server_init("remote_repo", address);
        });

        let client = thread::spawn(move||{
            let mut socket = TcpStream::connect(address).unwrap();
            socket.write("Hola server".as_bytes());
        });

        server.join().unwrap();
        client.join().unwrap();
    }
}