 use std::error::Error;

 use super::commands; 

pub fn command_handler(argv: Vec<String>) -> Result<(), Box<dyn Error>> {
    let command = argv[2].clone();

    if argv[1] != "gitr" {
        return Err("Usage: gitr <command> <args>".into());
    }

    let flags = argv[3..].to_vec();

    match command.as_str() {
        "hash-object" => commands::hash_object(flags),
        "cat-file" => commands::cat_file(flags),
        "init" => commands::init(flags),
        "status" => commands::status(flags),
        "add" => commands::add(flags),
        "rm" => commands::rm(flags),
        "commit" => commands::commit(flags),
        "checkout" => commands::checkout(flags),
        "log" => commands::log(flags),
        "clone" => commands::clone(flags),
        "fetch" => commands::fetch(flags),
        "merge" => commands::merge(flags),
        "remote" =>commands::remote(flags),
        "pull" => commands::pull(flags),
        "push" => commands::push(flags),
        "branch" =>commands::branch(flags),
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
            "target/debug/gitr".to_string(),
            "gitr".to_string(),
            "add".to_string(),
            "main.rs".to_string(),
        ];
        assert!(command_handler(argv).is_ok());
    }

    #[test]
    fn handler_error_input_incorrecto() {
        let argv = vec![
            "target/debug/gitr".to_string(),
            "no_gitr".to_string(),
            "add".to_string(),
            "main.rs".to_string(),
        ];
        assert!(command_handler(argv).is_err());
    }

    #[test]
    fn handler_error_comando_incorrecto() {
        let argv = vec![
            "target/debug/gitr".to_string(),
            "gitr".to_string(),
            "comando_no_existe".to_string(),
            "main.rs".to_string(),
        ];
        assert!(command_handler(argv).is_err());
    }
}