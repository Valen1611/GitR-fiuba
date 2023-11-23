use std::fs;

use gtk::{prelude::*, Application, Dialog, Entry, TextView, TextBuffer, ComboBoxText};

use gtk::{Builder,Window, Button, FileChooserButton};

use crate::commands::commands;
use crate::file_manager;

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
    
    
    //====Conexiones de señales====
    
    //====LOGIN====
    let connect_button_clone = connect_button.clone();
    let login_dialog_clone = login_dialog.clone();
    connect_button_clone.connect_clicked(move|_|{
        login_dialog_clone.show();
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
        commands::commit(cm_msg).unwrap();
    });

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
        let repo_branches = file_manager::get_branches().unwrap();
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
        commands::clone(vec![url.to_string(),"repo_clonado".to_string()]).unwrap();
        //Aca habria que setear el repo actual al recien
    });

    //====PUSH====
    let clone_push = push_button.clone();
    clone_push.connect_clicked(move|_|{
        let flags = vec![String::from("")];
        commands::push(flags).unwrap();
        println!("Push button clicked");
    });
    //====PULL====
    let clone_pull = pull_button.clone();
    clone_pull.connect_clicked(move|_|{
        let flags = vec![String::from("")];
        commands::pull(flags).unwrap();
        println!("Pull button clicked");
    });
    //====FETCH====
    let clone_fetch = fetch_button.clone();
    clone_fetch.connect_clicked(move|_|{
        let flags = vec![String::from("")];
        commands::fetch(flags);
        println!("Fetch button clicked");
    });

    window.set_application(Some(application));
    window.set_title("test");
     
    window.show_all();
    if !existe_config() {
        login_warning.show();
    }
    Some("Ok".to_string())
 }

pub fn initialize_gui(){
    let app = Application::builder()
        .application_id("org.example.HelloWorld")
        .build();

    app.connect_activate(|app| {
        build_ui(app);
    });
    app.run();
}
