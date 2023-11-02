use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use gtk::{prelude::*, Application, Dialog, Entry, TextView, TextBuffer, ComboBoxText};

use gtk::{Builder,Window, Button, FileChooserButton};

use crate::{file_manager, command_utils};

fn update_branches(branch_selector: &ComboBoxText,branches: Vec<String>){
    for branch in branches{
        branch_selector.append_text(&branch);
    }
}

fn build_ui(application: &gtk::Application)->Option<String>{
    let glade_src = include_str!("gui_test.glade");
    let builder = Builder::from_string(glade_src);

    let objetos = builder.objects();
    println!("{:?}",objetos);
   
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


    //====Conexiones de señales====
    let repo_url = Rc::new(RefCell::new(PathBuf::new()));
    let repo_url_clon = repo_url.clone();
    
    branch_selector.connect_changed(move|_|{
        buffer.set_text("mames que anda asi nomas");
    });
    //let valor = Rc::new(RefCell::new(0));

    repo_selector.connect_file_set(move |data|{
        *repo_url_clon.borrow_mut() = match data.filename(){
            Some(path) => path,
            None => return,
        }; //creo que esto es innecesario
        let data_a = data.filename().unwrap();
        let repo_name = data_a.file_name().unwrap().to_str().unwrap(); 
        println!("Repo name: {:?}", repo_name);
        file_manager::update_current_repo(&repo_name.to_string()).unwrap();
        let repo_branches = file_manager::get_branches().unwrap();
        println!("{:?}",repo_branches);
        update_branches(&branch_selector.clone(),repo_branches);

        //update current repo (existe) y le paso el nombre del repo.
        //print commits
        //
    });
    
    //let valor_clone = valor.clone();

    let clone_dialog_ = clone_dialog.clone();
    clone_button.connect_clicked(move|_| {
        clone_dialog_.show();
    });

    let clone_dialog_ = clone_dialog.clone();
    clone_accept_button.connect_clicked(move|_| {
        let url = clone_url.text();
        println!("Clonando repo: {:?}", url);
        clone_dialog_.hide();
    });

    

    window.set_application(Some(application));
    window.set_title("test");

    window.show_all();
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