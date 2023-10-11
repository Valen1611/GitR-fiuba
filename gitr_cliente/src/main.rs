use std::env;
mod commands;

fn main() {
    let argv: Vec<String> = env::args().collect();
    println!("argv: {:?}", argv);
    match commands::handler::command_handler(argv) {
        Ok(_) => println!("Handler Success"),
        Err(e) => println!("Handler Error: {}", e),
    };

}
