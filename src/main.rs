#![warn(clippy::all)]

extern crate gtk;
extern crate gio;

// To import all needed traits.
use gtk::prelude::*;
use gio::prelude::*;

use glib::clone;

use gtk::{Button, Builder, FileChooserDialog, FileFilter};

use std::env;

#[derive(Default)]
pub struct Header {
    container: gtk::HeaderBar
}

impl Header {
    pub fn new() -> Self {
        let container = gtk::HeaderBar::new();
        container.set_title(Some("Rust GlitchGUI by Sharky"));
        container.set_show_close_button(true);

        let btn_load = Button::new_from_icon_name(Some("document-new"), gtk::IconSize::Button);
        btn_load.set_widget_name("btn_load");
        container.add(&btn_load);

        let btn_save = Button::new_from_icon_name(Some("document-save"), gtk::IconSize::Button);
        btn_save.set_widget_name("btn_save");
        container.add(&btn_save);

        Header { container }
    }

    // oh my gaaaaaaaaawd this took entirely too long
    pub fn get_titlebar_button(&self, name: &str) -> Result<Button, gtk::Widget> {
        let children = self.container.get_children();
        for child in children {
            if child.get_widget_name().unwrap() == name {
                return child.downcast::<Button>();
            }
        }
        Err(self.container.clone().upcast::<gtk::Widget>())
    }
}

fn main() {
    let uiapp = gtk::Application::new(Some("pw.sharky.rust.glitchgui"),
                                      gio::ApplicationFlags::FLAGS_NONE)
                                 .expect("Application::new failed");
    uiapp.connect_activate(|app| {
        let builder = Builder::new_from_string(include_str!("main_window.glade"));

        let main_window: gtk::ApplicationWindow = builder.get_object(&"MainWindow").expect("Could not get MainWindow ?!");
        main_window.set_application(Some(app));
        main_window.set_resizable(true);
        main_window.resize(640, 480);

        // Set titlebar for dragging the window around
        let hdr_bar = Header::new();
        main_window.set_titlebar(Some(&hdr_bar.container));


        //let btn_load: gtk::Button = builder.get_object(&"btn_load").expect("Could not get btn_load?!");
        let btn_load = hdr_bar.get_titlebar_button("btn_load").expect("Could not get btn_load?!");
        btn_load.connect_clicked( move |_| {
            // todo: ask if we want to overwrite the already loaded image?

            let fcb = gtk::FileChooserDialogBuilder::new();
            let fc: FileChooserDialog = fcb.build();
            fc.set_action(gtk::FileChooserAction::Open);
            fc.add_buttons(&[
                (&"Open", gtk::ResponseType::Ok), (&"Cancel", gtk::ResponseType::Cancel)
            ]);

            let file_filter = FileFilter::new();
            file_filter.add_pattern(&"*.jpg");
            file_filter.add_pattern(&"*.jpg");
            fc.set_filter(&file_filter);

            if let gtk::ResponseType::Ok = fc.run() {
                // ok, load file
                //let source_image = gtk::Image::new_from_file(fc.get_filename().unwrap());

                let img_image: gtk::Image = builder.get_object(&"img_image").expect("Could not get img_image?!");
                img_image.set_from_file(fc.get_filename().unwrap());
            };
            fc.destroy();
            
        });



        main_window.show_all();
    });

    uiapp.run(&env::args().collect::<Vec<_>>());
}
