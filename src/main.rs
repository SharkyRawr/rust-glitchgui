#![warn(clippy::all)]

extern crate gtk;
extern crate gio;

// To import all needed traits.
use gtk::prelude::*;
use gio::prelude::*;

use glib::clone;

use gtk::{Builder, FileChooserDialog, FileFilter};

use std::env;

fn main() {
    let uiapp = gtk::Application::new(Some("pw.sharky.rust.glitchgui"),
                                      gio::ApplicationFlags::FLAGS_NONE)
                                 .expect("Application::new failed");
    uiapp.connect_activate(|app| {
        let builder = Builder::new_from_string(include_str!("main_window.glade"));

        let main_window: gtk::ApplicationWindow = builder.get_object(&"MainWindow").expect("Could not get MainWindow");
        main_window.set_application(Some(app));
        
        
        let btn_exit: gtk::Button = builder.get_object(&"btn_exit").expect("Could not get btn_exit?");
        btn_exit.connect_clicked(clone!(@weak main_window => move |_| {
            // todo: Ask if we are sure?
            main_window.close();
        }));


        let btn_load: gtk::Button = builder.get_object(&"btn_load").expect("Could not get btn_load?");
        btn_load.connect_clicked(clone!(@weak main_window => move |_| {
            // todo: ask if we want to overwrite the already loaded image?

            let fcb = gtk::FileChooserDialogBuilder::new();
            let _fc: FileChooserDialog = fcb.build();

            let _file_filter = FileFilter::new();
            
        }));



        main_window.show_all();
    });

    uiapp.run(&env::args().collect::<Vec<_>>());
}
