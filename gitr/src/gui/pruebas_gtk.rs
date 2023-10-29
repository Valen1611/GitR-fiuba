use gtk::ffi::GtkNotebook;
use gtk::{prelude::*, glib::BoolError, ApplicationWindow, Application};

use gtk::{Builder,Window, Button};
use gtk::builders::NotebookBuilder;


fn build_ui(application: &gtk::Application){
    let glade_src = include_str!("gui_test.glade");
    //println!("{}",glade_src);
    let builder = Builder::from_string(glade_src);

    let objetos = builder.objects();
    println!("{:?}",objetos);

    

    let window:Window = builder.object("main_window").unwrap();

    window.set_application(Some(application));
    window.set_title("test");

    window.show_all();

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

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_initialize_gui(){
        initialize_gui();
    }
}