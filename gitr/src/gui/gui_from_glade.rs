use std::fs;

use gtk::{prelude::*, Application, Dialog, Entry, TextView, TextBuffer, ComboBoxText};

use gtk::{Builder,Window, Button, FileChooserButton};

use crate::commands::commands;
use crate::file_manager;

fn get_commits() -> String{
    let mut commits = match  file_manager::commit_log("-1".to_string(),"client".to_string()) {
        Ok(commits) => commits,
        Err(_) => "No hay commits para mostrar".to_string(),
    };
    commits = commits.trim_end().to_string();

    let mut res = String::new();
    let max_string_len = 60;
    
    let mut fecha_actual = "-1";
    for mut commit in commits.split("\n\n\n").collect::<Vec<&str>>(){

        //println!("\x1b[34mCommit: \x1b[0m {:?}",commit);
        commit = commit.trim_start();
        let hash = commit.split('\n').collect::<Vec<&str>>()[0].split_at(8).1;
        let author = commit.split('\n').collect::<Vec<&str>>()[1].split_at(8).1;
        let date = commit.split('\n').collect::<Vec<&str>>()[2].split_at(5).1.trim_start();
        let message = commit.split('\n').collect::<Vec<&str>>()[3].trim_start();

        let day = date.split(' ').collect::<Vec<&str>>()[2];
        let time = date.split(' ').collect::<Vec<&str>>()[3];
        let hash_digits = hash.split_at(8).0;
        let short_message = if message.len() > 40 {
            message[..40].to_string() + "..."
        } else {
            message.to_string()
        };

        if day != fecha_actual {
            let month = date.split(' ').collect::<Vec<&str>>()[1];
            let year = date.split(' ').collect::<Vec<&str>>()[4];
            res.push_str("█\n");
            let fecha = format!("█■■> Commits on {} {}, {}\n", month, day, year);
            res.push_str(&fecha);
            res.push_str("█\n");
        }
        fecha_actual = day;
        let spaces_needed_first_line = max_string_len - short_message.len() - hash_digits.len();
        let spaces_needed_second_line = max_string_len - author.len() - time.len() - 3;
        res.push_str("█    ╔══════════════════════════════════════════════════════════════╗\n");
        res.push_str(&format!("█    ║ {}{: <width$}{} ║\n", short_message,"",  hash_digits, width = spaces_needed_first_line));
        res.push_str(&format!("█    ║ {}    {}{: <width$}║\n", author, time, "", width = spaces_needed_second_line));
        res.push_str("█    ╚══════════════════════════════════════════════════════════════╝\n");
    }

    res
}

#[cfg(test)]

mod tests {

    use super::*;

    #[test]
    fn print() {
        println!("{}", get_commits());
    }
}

fn update_branches(branch_selector: &ComboBoxText,branches: Vec<String>){
    branch_selector.remove_all();
    for branch in branches{
        branch_selector.append_text(&branch);
    }
}

fn existe_config() -> bool{
    fs::metadata("gitrconfig").is_ok()
}

fn build_ui(application: &gtk::Application,cliente: String)->Option<String>{
    let glade_src = include_str!("gui_test.glade");
    let builder = Builder::from_string(glade_src);
    let window:Window = builder.object("main_window")?;
    let repo_selector:FileChooserButton = builder.object("repo_selector")?;
    let clone_button: Button = builder.object("clone_button")?;
    let clone_dialog: Dialog = builder.object("clone_dialog")?;
    let clone_url: Entry = builder.object("clone_url")?;
    let clone_accept_button: Button = builder.object("clone_accept_button")?;
    let commit_text: TextView = builder.object("commit_text")?;
    let branch_selector: ComboBoxText = builder.object("branch_selector")?;
    let buffer: TextBuffer = commit_text.buffer()?;
    let commit: Button = builder.object("commit_button")?;
    let commit_dialog: Dialog = builder.object("commit_dialog")?;
    let commit_confirm: Button = builder.object("confirm_commit_button")?;
    let commit_message: Entry = builder.object("commit_message")?;
    let login_dialog: Window = builder.object("login_dialog")?;
    let login_warning: Dialog = builder.object("login_warning")?;
    let connect_button: Button = builder.object("connect_button")?;
    let login_button: Button = builder.object("login_button")?;
    let mail_entry: Entry = builder.object("mail_entry")?;
    let user_entry: Entry = builder.object("user_entry")?;
    let _connect_button_clone = connect_button.clone();
    let login_dialog_clone = login_dialog.clone();
    connect_button.connect_clicked(move|_|{
        login_dialog_clone.show();
    });

    let login_dialog_clone = login_dialog.clone();
    let login_button_clone = login_button.clone();
    login_button_clone.connect_clicked(move|_|{
        println!("Login button clicked");
        let mail = mail_entry.text();
        let name = user_entry.text();
        let config_file_data = format!("[user]\n\temail = {}\n\tname = {}\n", mail, name);
        file_manager::write_file("gitrconfig".to_string(), config_file_data).unwrap();
        login_dialog_clone.hide(); 
    });


    let commit_clone = commit.clone();
    let commit_dialog_clone = commit_dialog.clone();
    commit_clone.connect_clicked(move|_|{
        println!("Commit button clicked");
        commit_dialog_clone.show();
    }); 

    let commit_confirm_clone=commit_confirm.clone();
    let commit_dialog_clone = commit_dialog.clone();
    let cliente_clon = cliente.clone();

    commit_confirm_clone.connect_clicked(move|_|{
        commit_dialog_clone.close();
        commands::add(vec![".".to_string()],cliente_clon.clone()).unwrap();
        let message = format!("\"{}\"",commit_message.text());
        let cm_msg = vec!["-m".to_string(),message];
        commands::commit(cm_msg, "None".to_string() ,cliente_clon.clone()).unwrap();
    });

    let branch_selector_clon = branch_selector.clone();
    let cliente_clon = cliente.clone();
    branch_selector_clon.clone().connect_changed(move|_|{
        let branch = match branch_selector_clon.clone().active_text(){
            Some(branch) => branch,
            None => return,
        };
        let flags = vec![String::from(branch)];
        match commands::checkout(flags,cliente_clon.clone()){
            Ok(_) => (),
            Err(e) => {
                println!("Error al cambiar de branch: {:?}",e);
                return;
            },
        }
        let commits = file_manager::commit_log("-1".to_string(),cliente_clon.clone()).unwrap();
        buffer.set_text(&commits);
    });
    let cliente_clon = cliente.clone();
    let branch_selector_clon = branch_selector.clone();
    repo_selector.connect_file_set(move |data|{
        let data_a = data.filename().unwrap();
        let repo_name = data_a.file_name().unwrap().to_str().unwrap(); 
        println!("Repo name: {:?}", repo_name);
        file_manager::update_current_repo(&repo_name.to_string(),cliente_clon.clone()).unwrap();
        let repo_branches = file_manager::get_branches(cliente_clon.clone()).unwrap();
        println!("{:?}",repo_branches);
        update_branches(&branch_selector_clon.clone(),repo_branches);
    });

    let clone_dialog_ = clone_dialog.clone();
    clone_button.connect_clicked(move|_| {
        clone_dialog_.show();
    });

    let clone_dialog_ = clone_dialog.clone();
    clone_accept_button.connect_clicked(move|_| {
        let url = clone_url.text();
        println!("Clonando repo: {:?}", url);
        clone_dialog_.hide();
        commands::clone(vec![url.to_string(),"repo_clonado".to_string()],cliente.clone()).unwrap();
    });

    window.set_application(Some(application));
    window.set_title("test");
     
    window.show_all();
    if !existe_config() {
        login_warning.show();
    }
    Some("Ok".to_string())
 }

pub fn initialize_gui(cliente: String){
    let app = Application::builder()
        .application_id("org.example.HelloWorld")
        .build();
    let cliente_clon = cliente.clone();
    app.connect_activate(move|app| {
        build_ui(app, cliente_clon.clone());
    });
    app.run();
}