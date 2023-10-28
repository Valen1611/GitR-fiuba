extern crate flate2;
use gitr_cliente::git_transport::pack_file::*;

use std::{thread, net::{TcpStream, TcpListener}, io::{Write, Read}};
use flate2::write::ZlibEncoder;
use flate2::read::ZlibDecoder;
use std::error::Error;

use std::{env, io};
use std::io::{stdin,stdout};
use std::io::Write;




// fn main(){
//     let mut socket = TcpStream::connect("localhost:9418").unwrap();
//     let _ =socket.write("003cgit-upload-pack /mi-repo\0host=localhost:9418\0\0version=1\0".as_bytes());
//     println!("Envío git-upload-pack al daemon");

//     let mut buffer = [0;1024];
//     let mut bytes_read = socket.read(&mut buffer).expect("Error al leer socket");
        
//     println!("Leo {} bytes por el socket",bytes_read);


//     while bytes_read != 0{
//         let received_data = String::from_utf8_lossy(&buffer[..bytes_read]);
//         println!("String recibido: \n {}", received_data);
//         if received_data == "0000"{
//             println!("corto por recibir 0000");
//             break;
//         }
//         bytes_read = socket.read(&mut buffer).expect("Error al leer socket");
//         println!("Cantidad leida: {}",bytes_read);



    // println!("ok pasó el git-upload-pack, vemos el want");

    // let _ =socket.write("0032want cf6335a864bda2ee027ea7083a72d10e32921b15\n00000009done\n".as_bytes());
    // print!("ok mando want\n");
    
    // let mut buffer = [0;1024];

    // let mut bytes_read = socket.read(&mut buffer).expect("Error al leer socket");  //leo el ack  
    // println!("Leo {} bytes por el socket",bytes_read);
    // let received_data = String::from_utf8_lossy(&buffer);
    // // println!("String recibido: --{:?}--", received_data);

    // let mut bytes_read = socket.read(&mut buffer).expect("Error al leer socket"); //aca llega el packfile
    // println!("Leo {} bytes por el socket",bytes_read);
    // let received_data = String::from_utf8_lossy(&buffer);
    // println!("String recibido: --{:?}--", buffer);
    // PackFile::new_from_server_packfile(&mut buffer[..]).unwrap();
    fn main() {
    
        let mut input = String::new();
    
        while input != "q" {
    
            // Cuando tengamos la interfaz se deberia actualizar este mismo input supongo
            input = match get_input() {
                Ok(input) => input,
                Err(e) => {
                    println!("Error: {}", e);
                    break;
                }
            };
    
            let argv: Vec<String> = commands::handler::parse_input(input);
            
            // argv = ["command", "flag1", "flag2", ...]
            match commands::handler::command_handler(argv) {
                Ok(_) => println!("Handler Success"),
                Err(e) => {
                    match logger::log_error(e.to_string()) {
                        Ok(_) => (),
                        Err(e) => println!("Logger Error: {}", e),
                    };
                    println!("Handler Error: {}", e);}
            };
            input = String::new();
    
        }
    
}
/*



status (git man page) ✶✶✶

checkout (git man page) ✶✶✶✶✶

log (git man page)  ✶✶

clone (git man page)
fetch (git man page)
merge (git man page)
remote (git man page)
pull (git man page)
push (git man page)

*/
