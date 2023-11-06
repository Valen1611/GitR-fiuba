use gitr::{commands, logger, gitr_errors::GitrError, command_utils, file_manager, server};

use std::{io::{Write, self}, fs};
extern crate flate2;

// use gitr::gui::gui_from_glade::initialize_gui;




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
    fn get_input() -> Result<String, GitrError> {
        print!("\x1b[34mgitr: $ \x1b[0m");
        match io::stdout().flush() {
            Ok(_) => (),
            Err(e) => return Err(GitrError::InvalidArgumentError(e.to_string(), "Usage: TODO".to_string())),
        }
    
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => (),
            Err(e) => return Err(GitrError::InvalidArgumentError(e.to_string(), "Usage: TODO".to_string())),
        }
    
        Ok(input)
    }

    fn email_valido(email_recibido: String) -> bool {
        let email_parts:Vec<&str>  = email_recibido.split('@').collect::<Vec<&str>>();

        if email_parts.len() != 2 {
            return false; 
        }
        
        let domain = email_parts[1];

        if !domain.contains('.') {
            return false
        }

        true
    }

    fn setup_config_file(){
        let mut email_recibido = String::new();

        while !email_valido(email_recibido.clone()) {
            println!("Ingrese su email: ");
            email_recibido = match get_input() {
                Ok(email) => email,
                Err(_) => "user@mail.com".to_string(),
            };
        }
        println!("El email es valido, ya puede comenzar a usar Gitr");
        let name = command_utils::get_current_username();
        let config_file_data = format!("[user]\n\temail = {}\tname = {}\n", email_recibido, name);
        file_manager::write_file("gitrconfig".to_string(), config_file_data).unwrap();
        return;
    }

    fn existe_config() -> bool{
        fs::metadata("gitrconfig").is_ok()
    }

    fn print_bienvenida() {
        println!(        "\t╔══════════════════════════════════════════════╗");
        println!("\t║ \x1b[34mBienvenido a la version command-line de Gitr\x1b[0m ║");
        println!("\t║ \x1b[34mIntroduzca los comandos que desea realizar\x1b[0m   ║");
        println!("\t║ \x1b[34m(introduzca q para salir del programa)\x1b[0m       ║");
        println!(        "\t╚══════════════════════════════════════════════╝");
    }

    fn main() {
        //let child = std::thread::spawn(move || {
            // initialize_gui();
        //    server::server_init("repo_remoto", "localhost:9418")
        //});

        print_bienvenida();

        if !existe_config() {
            setup_config_file();
        }
        
    
        loop {
    
            // Cuando tengamos la interfaz se deberia actualizar este mismo input supongo
            let input = match get_input() {
                Ok(input) => input,
                Err(e) => {
                    println!("Error: {}", e);
                    break;
                }
            };
    
            if input == "q\n" {
                return;
            }

            let argv: Vec<String> = commands::handler::parse_input(input);
            
            // argv = ["command", "flag1", "flag2", ...]
            match commands::handler::command_handler(argv) {
                Ok(_) => (),
                Err(e) => {
                    println!("{}", e);
                    match logger::log_error(e.to_string()) {
                        Ok(_) => (),
                        Err(e) => println!("Logger Error: {}", e),
                    };
                }
            };

            
    
        }
        // match child.join(){
            // Ok(_) => (),
            // Err(e) => println!("Error al cerrar el thread de la GUI: {:?}",e),
        // }
    
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
