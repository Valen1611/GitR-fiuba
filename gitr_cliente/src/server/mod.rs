use std::io::Read;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::thread;

#[derive (Clone)]
struct GitrPaths {
    repo_path: String,
    socket_addr: String
}

pub fn init (r_path: &str, s_addr: &str) -> std::io::Result<()>  {
    let paths = &GitrPaths{repo_path: r_path.to_string(),socket_addr: s_addr.to_string()};
    let listener = TcpListener::bind(paths.socket_addr.clone())?;
    println!("Servidor escuchando en {}",paths.socket_addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {handle_client(stream,paths)});
            }
            Err(e) => {
                eprintln!("Error al aceptar la conexión: {}", e);
            }
        }
    }

    Ok(())
}
fn handle_client(mut stream: TcpStream, paths: &GitrPaths) {

    let mut buffer = [0; 1024];
    while let Ok(n) = stream.read(&mut buffer) {
        if n == 0 {
            // La conexión se cerró
            break;
        }
        if let Ok(request) = String::from_utf8(buffer[..n].to_vec()) {
            println!("Solicitud recibida: {}", request);

            // Hacer cosas con el request

            let response = format!("Eco: {}", request);
            stream.write(response.as_bytes()).unwrap();
        }
    }
}

