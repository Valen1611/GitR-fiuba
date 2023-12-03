use std::fs;

use gtk::gio::ApplicationFlags;
use gtk::{prelude::*, Application, Dialog, Entry, TextView, TextBuffer, ComboBoxText, Label};

use gtk::{Builder,Window, Button, FileChooserButton};

use crate::commands::commands::{self, remote};
use crate::file_manager;
use crate::gitr_errors::GitrError;

fn get_commits(cliente:String) -> String{
    let mut commits = match  file_manager::commit_log("-1".to_string(),cliente) {
        Ok(commits) => commits,
        Err(_) => return "No hay commits para mostrar".to_string(),
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

fn update_branches(branch_selector: &ComboBoxText,cliente: String){
    branch_selector.remove_all();
    let branches = match file_manager::get_branches(cliente.clone()){
        Ok(branches) => branches,
        Err(e) => {
            println!("Error al obtener branches: {:?}",e);
            //TODO WARNING DE QUE LA CARPETA NO ES UN REPO
            return;
        },
    };
    for branch in branches{
        branch_selector.append_text(&branch);
    }
}

fn existe_config(cliente: String) -> bool{
    fs::metadata(cliente.clone() +"/gitrconfig").is_ok()
}

fn build_ui(application: &gtk::Application, cliente: String)->Option<String>{
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
    let init_button: Button = builder.object("init_button")?;
    let init_dialog: Dialog = builder.object("init_dialog")?;
    let init_cancel_button: Button = builder.object("init_cancel_button")?;
    let init_accept_button: Button = builder.object("init_accept_button")?;
    let init_repo_name: Entry = builder.object("init_repo_name")?;
    let merge_branch_selector: ComboBoxText = builder.object("merge_branch_selector")?;
    let merge_button: Button = builder.object("merge_button")?;
    let add_branch_button: Button = builder.object("add_branch_button")?;
    let add_branch_dialog: Dialog = builder.object("add_branch_dialog")?;
    let branch_cancel_button: Button = builder.object("branch_cancel_button")?;
    let branch_button: Button = builder.object("branch_button")?;
    let new_branch_name: Entry = builder.object("new_branch_name")?;
    let conflict_file_chooser: FileChooserButton = builder.object("conflict_file_chooser")?;
    let conflict_text_view: TextView = builder.object("conflict_text_view")?;
    let conflict_buffer: TextBuffer = conflict_text_view.buffer()?;
    let conflict_save_button: Button = builder.object("conflict_save_button")?;
    let remote_error_label: Label = builder.object("remote_error_label")?;
    //====Conexiones de señales====
    //====ADD BRANCH====
    let add_branch_dialog_clone = add_branch_dialog.clone();
    add_branch_button.connect_clicked(move|_|{
        add_branch_dialog_clone.show();
    });

    let add_branch_dialog_clone = add_branch_dialog.clone();
    let branch_selector_clone = branch_selector.clone();
    let merge_branch_selector_clone = merge_branch_selector.clone();
    let cliente_ = cliente.clone();
    branch_button.connect_clicked(move|_|{
        let branch_name = new_branch_name.text();
        let flags = vec![branch_name.to_string()];
        match commands::branch(flags,cliente_.clone()){
            Ok(_) => (),
            Err(e) => {
                println!("Error al crear branch: {:?}",e);
                return;
            },
        };
        update_branches(&branch_selector_clone.clone(),cliente_.clone());
        update_branches(&merge_branch_selector_clone.clone(),cliente_.clone());
        add_branch_dialog_clone.hide();
    });

    branch_cancel_button.connect_clicked(move|_|{
        add_branch_dialog.hide();
    });


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
    let cliente_clon = cliente.clone();

    login_dialog_top_label.set_text(format!("Hola, {}. Por favor, ingrese su mail",cliente_clon.clone()).as_str());

    login_button_clone.connect_clicked(move|_|{
        println!("Login clicked");
        let mail = mail_entry.text().to_string();
        if !email_valido(mail.clone()){
            login_dialog_top_label.set_text("Mail inválido. Con formato nombre@xxxxxx.yyy");
            return;
        }
        let config_file_data = format!("[user]\n\temail = {}\n\tname = {}\n", mail, cliente_clon.clone());
        file_manager::write_file(cliente_clon.clone() + "/gitrconfig", config_file_data).unwrap();
        login_dialog_clone.hide();
    });

    let login_dialog_clone = login_dialog.clone();
    let login_warning_clone = login_warning.clone();
    login_connect_button.connect_clicked(move |_|{
        login_warning_clone.hide();
        login_dialog_clone.show();
    });

    //====LOGIN WARNING====
    let cliente_clone = cliente.clone();
    if !existe_config(cliente_clone.clone()) {
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
    let cliente_ = cliente.clone();
    let remote_error_dialog_clone = remote_error_dialog.clone();
    let remote_error_label_clone = remote_error_label.clone();
    let branch_selector_clone = branch_selector.clone();
    
    commit_confirm_clone.connect_clicked(move|_|{
        commit_dialog_clone.hide();
        match commands::add(vec![".".to_string()],cliente_.clone()){
            Ok(_)=> (),
            Err(e)=> {
                if e == GitrError::FileReadError(cliente_.clone()+"/.head_repo"){
                    remote_error_label_clone.set_text("No hay un repositorio asociado, busque o cree uno.");
                }
                else{
                    remote_error_label_clone.set_text(format!("Error al hacer add: {:?}",e).as_str());
                }
                remote_error_dialog_clone.show();
            },
        };
        let message = format!("\"{}\"",commit_message.text());
        let cm_msg = vec!["-m".to_string(),message];
        match commands::commit(cm_msg,"None".to_string(),cliente_.clone()){
            Ok(_) => (),
            Err(e) => {
                println!("Error al hacer commit: {:?}",e);
                return;
            },
        };
        
        update_branches(&branch_selector_clone, cliente_.clone());
    });

    close_commit_dialog_button.connect_clicked(move |_|{
        commit_dialog.hide();
        }
    );

    //====BRANCH====
    let branch_selector_clon = branch_selector.clone();
    let cliente_ = cliente.clone();

    branch_selector_clon.clone().connect_changed(move|_|{
        let branch = match branch_selector_clon.clone().active_text(){
            Some(branch) => branch,
            None => return,
        };
        let flags = vec![String::from(branch)];
        match commands::checkout(flags,cliente_.clone()){
            Ok(_) => (),
            Err(e) => {
                println!("Error al cambiar de branch: {:?}",e);
                return;
            },
        }
        let commits = get_commits(cliente_.clone());
        buffer.set_text(&commits);
    });

    let branch_selector_clon = branch_selector.clone();
    let merge_branch_selector_clon = merge_branch_selector.clone();
    let cliente_ = cliente.clone();
    let current_repo = match file_manager::get_current_repo(cliente_.clone()){
        Ok(repo) => {
            update_branches(&branch_selector_clon, cliente_.clone());
            update_branches(&merge_branch_selector_clon, cliente_.clone());
            repo},
        Err(e) => cliente_.clone(),
    };
    repo_selector.set_current_folder(current_repo.clone());
    repo_selector.connect_file_set(move |data|{
        let data_a = data.filename().unwrap();
        let repo_name = data_a.file_name().unwrap().to_str().unwrap(); 
        println!("Repo name: {:?}", repo_name);
        file_manager::update_current_repo(&repo_name.to_string(),cliente_.clone()).unwrap();
        
        update_branches(&branch_selector_clon.clone(),cliente_.clone());
        update_branches(&merge_branch_selector_clon.clone(),cliente_.clone());

    });

    //====CLONE====
    let clone_dialog_ = clone_dialog.clone();
    clone_button.connect_clicked(move|_| {
        clone_dialog_.show();
    });

    let clone_dialog_ = clone_dialog.clone();
    let cliente_ = cliente.clone();
    
    clone_accept_button.connect_clicked(move|_| {
        let url = clone_url.text();
        println!("Clonando repo: {:?}", url);
        clone_dialog_.hide();
        if commands::clone(vec![url.to_string(),"repo_clonado".to_string()],cliente_.clone()).is_err(){
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
    let cliente_ = cliente.clone();

    clone_push.connect_clicked(move|_|{
        let flags = vec![String::from("")];
        if commands::push(flags,cliente_.clone()).is_err(){
            println!("Error al hacer push");
            clone_error.show();
            return;
        };
        println!("Push button clicked");
    });
    //====PULL====
    let clone_pull = pull_button.clone();
    let clone_error = remote_error_dialog.clone();
    let cliente_ = cliente.clone();

    clone_pull.connect_clicked(move|_|{
        let flags = vec![String::from("")];
        if commands::pull(flags,cliente_.clone()).is_err(){
            println!("Error al hacer pull");
            clone_error.show();
            return;
        };
        println!("Pull button clicked");
    });
    //====FETCH====
    let clone_fetch = fetch_button.clone();
    let clone_error = remote_error_dialog.clone();
    let cliente_ = cliente.clone();
    clone_fetch.connect_clicked(move|_|{
        let flags = vec![String::from("")];
        if commands::fetch(flags,cliente_.clone()).is_err(){
            println!("Error al hacer fetch");
            clone_error.show();
            return;
        };
        println!("Fetch button clicked");
    });

    //====REMOTE ERROR DIALOG====
    let remote_error_dialog_clone=remote_error_dialog.clone();
    remote_error_close_button.connect_clicked(move|_|{
        remote_error_dialog_clone.hide();
    });

    //====INIT====
    let init_button_clone = init_button.clone();
    let init_dialog_clone = init_dialog.clone();
    init_button_clone.connect_clicked(move|_|{
        init_dialog_clone.show();
    });

    let init_dialog_clone = init_dialog.clone();
    init_cancel_button.connect_clicked(move|_|{
        init_dialog_clone.hide();
    });

    let init_dialog_clone = init_dialog.clone();
    let init_repo_name_clone = init_repo_name.clone();
    let cliente_ = cliente.clone();
    let repo_sel = repo_selector.clone();
    init_accept_button.connect_clicked(move|_|{
        let repo_name = init_repo_name_clone.text();
        init_dialog_clone.hide();
        if commands::init(vec![repo_name.to_string()],cliente_.clone()).is_err(){
            println!("Error al inicializar repo");
            return;
        };
        repo_sel.set_current_folder(cliente_.clone()+"/"+repo_name.as_str());
    });
    
    //====MERGE====
    let merge_button_clone = merge_button.clone();
    let merge_branch_selector_clone = merge_branch_selector.clone();
    let remote_error_dialog_clone = remote_error_dialog.clone();
    let remote_error_label_clone = remote_error_label.clone();

    let cliente_ = cliente.clone();
    merge_button_clone.connect_clicked(move|_|{
        let branch = match merge_branch_selector_clone.clone().active_text(){
            Some(branch) => branch,
            None => return,
        };
        let flags = vec![branch.to_string()];
        match commands::merge_(flags,cliente_.clone()){
            Ok(hubo_conflict) => {
                remote_error_label_clone.set_text("Surgieron conflicts al hacer merge, por favor arreglarlos y commitear el resultado.");
                remote_error_dialog_clone.show();
            },
            Err(e) => {
                println!("Error al hacer merge: {:?}",e);
                return;
            },
        }
    });

    //====CONFLICTS====
    let conflict_file_chooser_clone = conflict_file_chooser.clone();
    let cliente_ = cliente.clone();
    let conf_buffer = conflict_buffer.clone();
    conflict_file_chooser_clone.set_current_folder(cliente_.clone());
    conflict_file_chooser.connect_file_set(move |data|{
        let filename = data.filename().unwrap().to_str().unwrap().to_string();
        let data_from_file = file_manager::read_file(filename.clone()).unwrap();
        println!("Data from file: {:?}",data_from_file);
        conf_buffer.set_text(&data_from_file);
    });

    let conf_buffer = conflict_buffer.clone();
    conflict_save_button.connect_clicked(move|_|{
        let filename = conflict_file_chooser_clone.filename().unwrap().to_str().unwrap().to_string();
        let data = conf_buffer.text(&conf_buffer.start_iter(),&conf_buffer.end_iter(),false).unwrap().to_string();
        file_manager::write_file(filename.clone(),data).unwrap();
    });

    window.set_application(Some(application));
    window.set_title("test");
    window.show_all();
    Some("Ok".to_string())
 }

 pub fn initialize_gui(cliente: String){
    let app = Application::new(Some("test.test"), ApplicationFlags::HANDLES_OPEN);
    let cliente_clon = cliente.clone();
    app.connect_open(move|app,files,_| {
        build_ui(&app, cliente_clon.clone());
    });

    app.run();
}
