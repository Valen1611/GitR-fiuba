use std::fs;

use gtk::{prelude::*, Application, Dialog, Entry, TextView, TextBuffer, ComboBoxText, Label};

use gtk::{Builder,Window, Button, FileChooserButton};

use crate::commands::commands::{self, remote};
use crate::file_manager;

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

fn update_branches(branch_selector: &ComboBoxText,branches: Vec<String>){
    branch_selector.remove_all();
    for branch in branches{
        branch_selector.append_text(&branch);
    }
}

fn existe_config() -> bool{
    fs::metadata("gitrconfig").is_ok()
}

fn build_ui(application: &gtk::Application)->Option<String>{
    let glade_src = include_str!("gui_test.glade");
    let builder = Builder::from_string(glade_src);
   
    //====Builders para componentes====
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
    let push_button: Button = builder.object("push_button")?;
    let pull_button: Button = builder.object("pull_button")?;
    let fetch_button: Button = builder.object("fetch_button")?;
    let remote_error_dialog: Dialog = builder.object("remote_error")?;
    let remote_error_close_button: Button = builder.object("remote_error_close_button")?;
    let close_commit_dialog_button: Button = builder.object("close_commit_dialog_button")?;
    let cancel_clone_button: Button = builder.object("cancel_clone_button")?;
    let cancel_login_button: Button = builder.object("cancel_login_button")?;
    let login_close_button: Button = builder.object("login_close_button")?;
    let login_connect_button: Button = builder.object("login_connect_button")?;
    let login_dialog_top_label: Label = builder.object("login_dialog_top_label")?;
    //====Conexiones de señales====
    
    //====LOGIN====
    let connect_button_clone = connect_button.clone();
    let login_dialog_clone = login_dialog.clone();
    connect_button_clone.connect_clicked(move|_|{
        login_dialog_clone.show();
    });

    let login_dialog_clone = login_dialog.clone();
    cancel_login_button.connect_clicked(move|_|{
        login_dialog_clone.hide();
    });

    let login_button_clone = login_button.clone();
    let login_dialog_clone = login_dialog.clone();
    login_button_clone.connect_clicked(move|_|{
        println!("Login clicked");
        let mail = mail_entry.text().to_string();
        let user = user_entry.text().to_string();
        if !email_valido(mail.clone()){
            login_dialog_top_label.set_text("Mail inválido. Con formato nombre@xxxxxx.yyy");
            return;
        }
        if user.is_empty(){
            login_dialog_top_label.set_text("Usuario vacío. Ingrese un usuario válido");
            return;
        }
        let config_file_data = format!("[user]\n\temail = {}\n\tname = {}\n", mail, user);
        file_manager::write_file("gitrconfig".to_string(), config_file_data).unwrap();
        login_dialog_clone.hide();
    });

    let login_dialog_clone = login_dialog.clone();
    let login_warning_clone = login_warning.clone();
    login_connect_button.connect_clicked(move |_|{
        login_warning_clone.hide();
        login_dialog_clone.show();
    });

    //====LOGIN WARNING====
    if !existe_config() {
        login_warning.show();
    }

    login_close_button.connect_clicked(move |_|{
        login_warning.hide();
    });

    //====COMMIT====
    let commit_clone = commit.clone();
    let commit_dialog_clone = commit_dialog.clone();
    commit_clone.connect_clicked(move|_|{
        println!("Commit button clicked");
        commit_dialog_clone.show();
    }); 

    let commit_confirm_clone=commit_confirm.clone();
    let commit_dialog_clone = commit_dialog.clone();
    commit_confirm_clone.connect_clicked(move|_|{
        commit_dialog_clone.close();
        commands::add(vec![".".to_string()]).unwrap();
        let message = format!("\"{}\"",commit_message.text());
        let cm_msg = vec!["-m".to_string(),message];
        if commands::commit(cm_msg).is_err(){
            println!("Error al hacer commit");
            return;
        };
    });

    close_commit_dialog_button.connect_clicked(move |_|{
        commit_dialog.hide();
        }
    );

    //====BRANCH====
    let branch_selector_clon = branch_selector.clone();
    branch_selector_clon.clone().connect_changed(move|_|{
        let branch = match branch_selector_clon.clone().active_text(){
            Some(branch) => branch,
            None => return,
        };
        let flags = vec![String::from(branch)];
        match commands::checkout(flags){
            Ok(_) => (),
            Err(e) => {
                println!("Error al cambiar de branch: {:?}",e);
                return;
            },
        }
        let commits = file_manager::commit_log("-1".to_string()).unwrap();
        buffer.set_text(&commits);
    });

    let branch_selector_clon = branch_selector.clone();
    repo_selector.connect_file_set(move |data|{
        let data_a = data.filename().unwrap();
        let repo_name = data_a.file_name().unwrap().to_str().unwrap(); 
        println!("Repo name: {:?}", repo_name);
        file_manager::update_current_repo(&repo_name.to_string()).unwrap();
        let repo_branches = match(file_manager::get_branches()){
            Ok(branches) => branches,
            Err(e) => {
                println!("Error al obtener branches: {:?}",e);
                return;
            },
        };
        println!("{:?}",repo_branches);
        update_branches(&branch_selector_clon.clone(),repo_branches);
    });

    //====CLONE====
    let clone_dialog_ = clone_dialog.clone();
    clone_button.connect_clicked(move|_| {
        clone_dialog_.show();
    });

    let clone_dialog_ = clone_dialog.clone();
    clone_accept_button.connect_clicked(move|_| {
        let url = clone_url.text();
        println!("Clonando repo: {:?}", url);
        clone_dialog_.hide();
        if commands::clone(vec![url.to_string(),"repo_clonado".to_string()]).is_err(){
            println!("Error al clonar");
            return;
        };
        //Aca habria que setear el repo actual al recien con el selector y el file explorer
    });

    cancel_clone_button.connect_clicked(move|_|{
        clone_dialog.hide();
    });

    //====PUSH====
    let clone_push = push_button.clone();
    let clone_error = remote_error_dialog.clone();
    clone_push.connect_clicked(move|_|{
        let flags = vec![String::from("")];
        if commands::push(flags).is_err(){
            println!("Error al hacer push");
            clone_error.show();
            return;
        };
        println!("Push button clicked");
    });
    //====PULL====
    let clone_pull = pull_button.clone();
    let clone_error = remote_error_dialog.clone();
    clone_pull.connect_clicked(move|_|{
        let flags = vec![String::from("")];
        if commands::pull(flags).is_err(){
            println!("Error al hacer pull");
            clone_error.show();
            return;
        };
        println!("Pull button clicked");
    });
    //====FETCH====
    let clone_fetch = fetch_button.clone();
    let clone_error = remote_error_dialog.clone();
    clone_fetch.connect_clicked(move|_|{
        let flags = vec![String::from("")];
        if commands::fetch(flags).is_err(){
            println!("Error al hacer fetch");
            clone_error.show();
            return;
        };
        println!("Fetch button clicked");
    });

    

    //====REMOTE ERROR DIALOG====
    remote_error_close_button.connect_clicked(move|_|{
        remote_error_dialog.hide();
    });

    window.set_application(Some(application));
    window.set_title("test");
    window.show_all();
    Some("Ok".to_string())
 }

pub fn initialize_gui(){
    let app = Application::builder()
        .application_id("org.gitr.gui")
        .build();

    app.connect_activate(|app| {
        build_ui(app);
    });
    app.run();
}
