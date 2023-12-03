use crate::{logger, gitr_errors::GitrError};
use super::commands; 

pub fn parse_input(input: String) -> Vec<String> {
    return input.split_whitespace().map(|s| s.to_string()).collect();
}

/// ["command", "flag1", "flag2", ...]
pub fn command_handler(argv: Vec<String>,  hubo_conflict: &mut bool , branch_name: &mut String, client: String) -> Result<bool, GitrError> {

    if argv.is_empty() {
        return Ok(false)
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
        "add" => commands::add(flags,client)?,
        "rm" => commands::rm(flags,client)?,
        "commit" => {
        if *hubo_conflict {
            println!("hubo conflict, branch name: {}", branch_name);
            commands::commit(flags, branch_name.clone(), client)?;
            *hubo_conflict = false;
            *branch_name = "".to_string();
            
        } else {
            commands::commit(flags, "None".to_string(), client)?;
        }

        },
        "checkout" => commands::checkout(flags,client)?,
        "log" => commands::log(flags,client)?,
        "clone" => commands::clone(flags,client)?,
        "fetch" => commands::fetch(flags,client)?,
        "merge" => return Ok(commands::merge_(flags,client)?),
        "remote" =>commands::remote(flags,client)?,
        "pull" => commands::pull(flags,client)?,
        "push" => commands::push(flags,client)?,
        "branch" =>commands::branch(flags,client)?,
        "ls-files" => commands::ls_files(flags,client)?,
        "show-ref" => commands::show_ref(flags,client)?,
        "tag" => commands::tag(flags,client)?,
        "ls-tree" => commands::ls_tree(flags,client)?,
        "rebase" => commands::rebase(flags,client)?,
        "q" => return Ok(false),
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

    Ok(false)

}
