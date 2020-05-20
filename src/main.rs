#![warn(clippy::all)]

extern crate gdk_pixbuf;
extern crate gio;
extern crate gtk;
extern crate rand;

// To import all needed traits.
use gdk_pixbuf::prelude::*;
use gio::prelude::*;
use gtk::prelude::*;
use rand::prelude::*;

use glib::clone;

use gdk_pixbuf::Pixbuf;
use gtk::{Builder, Button, FileChooserDialog, FileFilter};

use std::env;
use std::io::Read;

#[derive(Default)]
pub struct Header {
    container: gtk::HeaderBar,
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

        let btn_num = gtk::SpinButton::new_with_range(0f64, 1000f64, 1f64);
        btn_num.set_input_purpose(gtk::InputPurpose::Number);
        btn_num.set_widget_name("btn_num");
        container.add(&btn_num);

        Header { container }
    }

    // oh my gaaaaaaaaawd this took entirely too long
    pub fn get_titlebar_button<T: IsA<gtk::Widget>>(&self, name: &str) -> Result<T, gtk::Widget> {
        let children = self.container.get_children();
        for child in children {
            if child.get_widget_name().unwrap() == name {
                return child.downcast::<T>();
            }
        }
        Err(self.container.clone().upcast::<gtk::Widget>())
    }
}

fn glitch_imagefile_by_numbytes(path: &std::path::PathBuf, numbytes: u32) -> gdk_pixbuf::Pixbuf {
    let mut rng = rand::thread_rng();
    let file = std::fs::File::open(path).unwrap();
    let mut br = std::io::BufReader::new(file);
    let mut bytes: Vec<u8> = Vec::new();
    let _num_bytes = br.read_to_end(&mut bytes).unwrap();

    //let orig_image_bytes = bytes.clone(); // for later

    let glitch_amount = 1024;
    let bytes: Vec<u8> = bytes
        .iter()
        .map(|orig_byte| {
            if rng.next_u32() % glitch_amount == 0 {
                (rng.next_u32() % 256) as u8
            } else {
                *orig_byte
            }
        })
        .collect();

    /*
    for _ in 0..numbytes {
        let rnd = rng.next_u32() % bytes.len() as u32;
        bytes[rnd as usize] = (rng.next_u32() % 256) as u8;
    }
    */

    let glib_bytes = glib::Bytes::from_owned(bytes);
    let instream = gio::MemoryInputStream::new_from_bytes(&glib_bytes);

    let not_cancellable: Option<&gio::Cancellable> = None;
    match gdk_pixbuf::Pixbuf::new_from_stream(&instream, not_cancellable) {
        Ok(a) => a,

        Err(e) => panic!(e.to_string()), // todo repeat until parseable
    }
}

fn main() {
    let uiapp = gtk::Application::new(
        Some("pw.sharky.rust.glitchgui"),
        gio::ApplicationFlags::FLAGS_NONE,
    )
    .expect("Application::new failed");
    uiapp.connect_activate(|app| {
        let builder: Builder = Builder::new_from_string(include_str!("main_window.glade"));

        let main_window: gtk::ApplicationWindow = builder
            .get_object(&"MainWindow")
            .expect("Could not get MainWindow ?!");
        main_window.set_application(Some(app));
        main_window.set_resizable(true);
        main_window.resize(640, 480);

        // Set titlebar for dragging the window around
        let hdr_bar = Header::new();
        main_window.set_titlebar(Some(&hdr_bar.container));

        //The problem: GTK gives you a value in a callback that could be running on a diff thread
        //Also, as a callback happens *eventually*, the code that happens right after the end of btn_load.connect_clicked
        //Doesn't necessarily have filename set, even if you could do that. (it probably won't)
        //Move semantics say you can take something from outside, and permanently move it inside the closure
        //We're wanting to do the opposite.
        //Options:
        // 1. Actor model (callback sends an event to something that knows how to perform the operation)
        //    (this is the fancy one and i wouldn't recommend it but i think it's cool)
        // 2. Just start the operation from inside the callback
        //    (the easy option)
        // 3. Have some amount of global shared mutable memory, with an Arc<Mutex<T>>
        //    (the c developer option)

        // move fixes that, just need a reference of some type inside the closure and clone does that ( ithink)
        let btn_load: Button = hdr_bar
            .get_titlebar_button("btn_load")
            .expect("Could not get btn_load?!");
        btn_load.connect_clicked(move |_| {
            // todo: ask if we want to overwrite the already loaded image?

            let fcb = gtk::FileChooserDialogBuilder::new();
            let fc: FileChooserDialog = fcb.build();
            fc.set_action(gtk::FileChooserAction::Open);
            fc.add_buttons(&[
                (&"Open", gtk::ResponseType::Ok),
                (&"Cancel", gtk::ResponseType::Cancel),
            ]);

            let file_filter = FileFilter::new();
            file_filter.add_pattern(&"*.jpg");
            file_filter.add_pattern(&"*.jpg");
            fc.set_filter(&file_filter);

            if let gtk::ResponseType::Ok = fc.run() {
                // ok, load file
                //filename = fc.get_filename().unwrap();

                let img_image: gtk::Image = builder
                    .get_object(&"img_image")
                    .expect("Could not get img_image?!");
                let btn_num: gtk::SpinButton = hdr_bar.get_titlebar_button("btn_num").unwrap();
                //img_image.set_from_file(fc.get_filename().unwrap());
                let glitched_buf = glitch_imagefile_by_numbytes(
                    &fc.get_filename().unwrap(),
                    btn_num.get_value_as_int() as u32,
                );
                img_image.set_from_pixbuf(Some(&glitched_buf));
            };
            fc.destroy();
        });
        /*
                let btn_num: gtk::SpinButton = hdr_bar.get_titlebar_button("btn_num").expect("Could not get btn_num?!");
                btn_num.connect_changed(clone!(@strong filename => move |me| {
                    let value = me.get_value_as_int();

                    let glitched_buf = glitch_imagefile_by_numbytes(&filename, value as u32);
                    let img_image: gtk::Image = builder.get_object(&"img_image").expect("Could not get img_image?!");
                    img_image.set_from_pixbuf(Some(&glitched_buf));
                }));
        */

        main_window.show_all();
    });

    uiapp.run(&env::args().collect::<Vec<_>>());
}















