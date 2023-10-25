use std::error::Error;
use std::{env, io};
mod commands;
mod objects;
mod file_manager;
mod gitr_errors;
use std::io::{stdin,stdout,Write};
mod command_utils;



fn get_input() -> Result<String, Box<dyn Error>> {
    print!("gitr: $ ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input)
}

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
            Err(e) => println!("Handler Error: {}", e),
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