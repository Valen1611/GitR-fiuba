extern crate flate2;
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
    let listener = TcpListener::bind(s_addr)?;
    let mut childs = Vec::new();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
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
    while let Ok(n) = stream.read(&mut buffer) {
        if n == 0 {
            // La conexión se cerró
            break;
        }
        if let Ok(request) = decode(&buffer) { 

            // Hacer cosas con el request
            let reply = server_handler(request, &r_path)?;

            if let Ok(reply) = code(&reply){
                stream.write(&reply)?;

            } 
        }
    }
    Ok(())
}

fn server_handler(request: Vec<u8>, r_path: &str) -> std::io::Result<Vec<u8>> {

    // ej: 003egit-upload-pack/project.git\0host=myserver.com\0
    let pkt_line = from_utf8(&request).unwrap_or(""); 
    is_valid_pkt_line(pkt_line)?;
    let elems = split_n_validate_elems(pkt_line)?;


    Ok(request)
}

fn is_valid_pkt_line(pkt_line: &str) -> std::io::Result<()> {
    if pkt_line == ""|| pkt_line.len() <= 4 || usize::from_str_radix(pkt_line.split_at(4).0,16) != Ok(pkt_line.len()) {
        return Err(Error::new(std::io::ErrorKind::ConnectionRefused, "Error: No se sigue el estandar de PKT-LINE"))
    }
    Ok(())
}

fn split_n_validate_elems(pkt_line: &str) -> std::io::Result<Vec<&str>> {
    let line = pkt_line.split_at(4).1;
    let div1: Vec<&str> = line.split("/").collect(); // [comando , resto] 
    let div2: Vec<&str> = div1[1].split("\'").collect(); // [repo_local_path, "0" + gitr.com???, "0"]
    let mut elems: Vec<&str> = vec![];
    if (div1.len() != 2) || div2.len()!= 3 {
        return Err(Error::new(std::io::ErrorKind::ConnectionRefused, "Error: No se sigue el estandar de PKT-LINE"))
    }
    elems.push(div1[0]);
    elems.push(div2[0]);
    if let Some(host) = div2[1].strip_prefix("0") {
        elems.push(host);
        return Ok(elems)
    }
    Err(Error::new(std::io::ErrorKind::ConnectionRefused, "Error: No se sigue el estandar de PKT-LINE"))
}

fn create_dirs(r_path: &str) -> std::io::Result<()> {
    let p_str = r_path.to_string();
    fs::create_dir(p_str.clone() + "objects")?;
    fs::create_dir(p_str.clone() + "refs")?;
    fs::create_dir(p_str.clone() +"/refs/heads")?;
    fs::create_dir(p_str.clone() +"/refs/remotes")?;
    fs::create_dir(p_str.clone() +"/refs/remotes/origin")?;
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
