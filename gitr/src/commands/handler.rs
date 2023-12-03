use crate::{logger, gitr_errors::GitrError};
use super::commands; 

pub fn parse_input(input: String) -> Vec<String> {
    return input.split_whitespace().map(|s| s.to_string()).collect();
}

/// ["command", "flag1", "flag2", ...]
pub fn command_handler(argv: Vec<String>,  hubo_conflict:  bool , branch_hash: String, client: String) -> Result<(bool, String), GitrError> {

    if argv.is_empty() {
        return Ok((false, "".to_string()))
    }

    let command = argv[0].clone();

    let flags = argv[1..].to_vec();
    
    
    let message = format!("calling {} with flags: {:?}", command, flags);
    match logger::log_action(message.clone()) {
        Ok(_) => (),
        Err(e) => println!("Error: {}", e),
    };
    
    match command.as_str() {
        "hash-object" | "h" => commands::hash_object(flags,client)?, //"h" para testear mas rapido mientras la implementamos
        "cat-file" | "c" => commands::cat_file(flags,client)?,
        "init" => commands::init(flags,client)?,
        "status" => commands::status(flags,client)?,
        "add" => {
            commands::add(flags,client)?;
            return Ok((hubo_conflict, branch_hash));
        },
        "rm" => commands::rm(flags,client)?,
        "commit" => {
        if hubo_conflict {
            commands::commit(flags, branch_hash.clone(), client)?;
            return Ok((false, "".to_string()));
        } else {
            commands::commit(flags, "None".to_string(), client)?;
        }

        },
        "checkout" => commands::checkout(flags,client)?,
        "log" => commands::log(flags,client)?,
        "clone" => commands::clone(flags,client)?,
        "fetch" => commands::fetch(flags,client)?,
        "merge" => {
            let (hubo_conflict_res, branch_hash_res) = commands::merge_(flags,client)?;
            if hubo_conflict_res {
                println!("\x1b[33mHubo un conflicto, por favor resuelvalo antes de continuar\x1b[0m");
            }
            return Ok((true, branch_hash_res));
        },
        "remote" =>commands::remote(flags,client)?,
        "pull" => commands::pull(flags,client)?,
        "push" => commands::push(flags,client)?,
        "branch" =>commands::branch(flags,client)?,
        "ls-files" => commands::ls_files(flags,client)?,
        "show-ref" => commands::show_ref(flags,client)?,
        "tag" => commands::tag(flags,client)?,
        "ls-tree" => commands::ls_tree(flags,client)?,
        "rebase" => commands::rebase(flags,client)?,
        "check-ignore" => commands::check_ignore(flags,client)?,
        "q" => return Ok((false, "".to_string())),
        "l" => logger::log(flags)?,
        "list-repos" | "lr" => commands::list_repos(client),
        "go-to-repo" | "gtr" => commands::go_to_repo(flags,client)?,
        "cur-repo" | "cr" => commands::print_current_repo(client)?,
        "echo" => commands::echo(flags,client)?,
        _ => {
            let message = format!("invalid command: {}", command);
            return Err(GitrError::InvalidArgumentError(message, "usage: gitr <command> [<args>]".to_string()));
        }
    }

    Ok((false, "".to_string()))

}
