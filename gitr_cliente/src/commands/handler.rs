use std::error::Error;


use crate::logger;

use super::commands; 

pub fn parse_input(input: String) -> Vec<String> {
    return input.split_whitespace().map(|s| s.to_string()).collect();
}

/// ["command", "flag1", "flag2", ...]
pub fn command_handler(argv: Vec<String>) -> Result<(), Box<dyn Error>> {
    let command = argv[0].clone();

    let flags = argv[1..].to_vec();
    
    let message = format!("calling {} with flags: {:?}", command, flags);
    logger::log_action(message.clone())?;
    match command.as_str() {
        "hash-object" | "h" => commands::hash_object(flags)?, //"h" para testear mas rapido mientras la implementamos
        "cat-file" | "c" => commands::cat_file(flags)?,
        "init" => commands::init(flags)?,
        "status" => commands::status(flags),
        "add" => commands::add(flags)?,
        "rm" => commands::rm(flags)?,
        "commit" => commands::commit(flags)?,
        "checkout" => commands::checkout(flags)?,
        "log" => commands::log(flags),
        "clone" => commands::clone(flags),
        "fetch" => commands::fetch(flags),
        "merge" => commands::merge(flags),
        "remote" =>commands::remote(flags),
        "pull" => commands::pull(flags),
        "push" => commands::push(flags),
        "branch" =>commands::branch(flags)?,
        "ls-files" => commands::ls_files(flags),
        "q" => return Ok(()),
        "l" => logger::log(flags)?,
        _ => {
            let message = format!("invalid command: {}", command);
            return Err(message.into());
        }
    }

    Ok(())

}

// Para comandos para cosas del server, se podria hacer otra funcion




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handler_funciona_input_correcto() {
        let argv = vec![
            "log".to_string(),
            "main.rs".to_string(),
        ];
        assert!(command_handler(argv).is_ok());
    }

    #[test]
    fn handler_error_comando_incorrecto() {
        let argv = vec![
            "comando_no_existe".to_string(),
            "main.rs".to_string(),
        ];
        assert!(command_handler(argv).is_err());
    }
}