use gitr::{commands, logger, gitr_errors::GitrError, command_utils, file_manager, server};

use std::{io::{Write, self}, fs};
extern crate flate2;

use gitr::gui::gui_from_glade::initialize_gui;

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
        let child = std::thread::spawn(move || {
            initialize_gui();
            server::server_init("repo_remoto", "localhost:9418")
        });

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
        match child.join(){
            Ok(_) => (),
            Err(e) => println!("Error al cerrar el thread de la GUI: {:?}",e),
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
