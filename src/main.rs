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

//use glib::clone;

//use gdk_pixbuf::Pixbuf;
use gtk::{Builder, Button, FileChooserDialog, FileFilter};

use std::env;
use std::io::Read;
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct Header {
    container: gtk::HeaderBar,
}

impl Header {
    pub fn new() -> Self {
        let container = gtk::HeaderBar::new();
        container.set_title(Some("Rust GlitchGUI by Sharky & 0xADD1E"));
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

fn glitch_imagefile_by_numbytes(
    path: &std::path::PathBuf,
    num_glitches: u32,
) -> Result<gdk_pixbuf::Pixbuf, gdk_pixbuf::PixbufError> {
    let mut rng = rand::thread_rng();
    let file = std::fs::File::open(path).unwrap();
    let mut br = std::io::BufReader::new(file);
    let mut bytes: Vec<u8> = Vec::new();
    let _num_bytes = br.read_to_end(&mut bytes).unwrap();
    let orig_image_bytes = bytes.clone();

    let mut tries: u32 = 0;
    'parse_loop: loop {
        bytes.clear();
        bytes.clone_from(&orig_image_bytes); // reset source image

        for _ in 0..num_glitches {
            let glitch_index = (rng.next_u32() as usize % bytes.len()) as usize;
            // 💋
            // no u

            bytes[glitch_index] = (rng.next_u32() % 256) as u8;
        }

        // Read bytes as MemoryInputStream and try to parse a Pixbuf from it
        let glib_bytes = glib::Bytes::from(&bytes);
        let instream = gio::MemoryInputStream::new_from_bytes(&glib_bytes);
        let not_cancellable: Option<&gio::Cancellable> = None;
        match gdk_pixbuf::Pixbuf::new_from_stream(&instream, not_cancellable) {
            Ok(a) => return Ok(a),

            Err(e) => {
                println!("{:?}", e);
                if tries > 20 {
                    break 'parse_loop;
                }
            }
        }
        tries += 1;
    }
    println!("Giving up after 20 tries :(");
    Err(gdk_pixbuf::PixbufError::Failed)
}

fn save_pixbuf(
    pixbuf: &gdk_pixbuf::Pixbuf,
    path: &std::path::PathBuf,
) -> Result<(), glib::error::Error> {
    let filetype = match path.extension() {
        Some(e) => e.to_str().unwrap().replace("jpg", "jpeg"),
        None => "jpeg".to_string(),
    };
    let mut path = std::path::PathBuf::from(path);
    path.set_extension(&filetype);
    // todo: can we automate this from the FileChooser filter??

    //let filetype = path.extension().unwrap().to_str().unwrap().to_string().replace("jpg", "jpeg");
    let empty_options: &[(&str, &str)] = &[("", "")];
    pixbuf.savev(path, &filetype, empty_options)
}

fn main() {
    let uiapp = gtk::Application::new(
        Some("pw.sharky.rust.glitchgui"),
        gio::ApplicationFlags::FLAGS_NONE,
    )
    .expect("Application::new failed");
    uiapp.connect_activate(|app| {
        let builder: Builder = Builder::new_from_string(include_str!("main_window.glade"));
        let arc_builder = Arc::new(Mutex::new(builder));

        let main_window: gtk::ApplicationWindow = arc_builder.lock().unwrap().get_object(&"MainWindow").expect("Could not get MainWindow ?!");
        main_window.set_application(Some(app));
        main_window.set_resizable(true);
        main_window.resize(640, 480);

        // Set titlebar for dragging the window around
        let hdr_bar = Header::new();
        let arc_hdr_bar = Arc::new(Mutex::new(hdr_bar));
        main_window.set_titlebar(Some(&arc_hdr_bar.lock().unwrap().container));

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
        // 3. Have some amount of global shared mutable memory, with an Arc<Mutex<T>> 👈
        //    (the c developer option)  👈

        
        let arc_filename = Arc::new(Mutex::new(String::new()));

        {
            let arc_builder = arc_builder.clone();
            let arc_filename = arc_filename.clone();
            let arc_hdr_bar = arc_hdr_bar.clone();
        
            let btn_load: Button = arc_hdr_bar.lock().unwrap().get_titlebar_button("btn_load").expect("Could not get btn_load?!");
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
                file_filter.set_name(Some("All supported image files"));
                file_filter.add_pattern(&"*.jpg");
                file_filter.add_pattern(&"*.jpeg");
                file_filter.add_pattern(&"*.png");
                file_filter.add_pattern(&"*.gif");
                fc.add_filter(&file_filter);

                let file_filter = FileFilter::new();
                file_filter.set_name(Some("JPEG images (*.jpg)"));
                file_filter.add_pattern(&"*.jpg");
                file_filter.add_pattern(&"*.jpeg");
                fc.add_filter(&file_filter);

                let file_filter = FileFilter::new();
                file_filter.set_name(Some("PNG images (*.png)"));
                file_filter.add_pattern(&"*.png");
                fc.add_filter(&file_filter);

                let file_filter = FileFilter::new();
                file_filter.set_name(Some("GIF images (*.gif)"));
                file_filter.add_pattern(&"*.gif");
                fc.add_filter(&file_filter);

                if let gtk::ResponseType::Ok = fc.run() {
                    // ok, load file
                    *arc_filename.lock().unwrap() = fc.get_filename().unwrap().to_str().unwrap().to_string();

                    let img_image: gtk::Image = arc_builder.lock().unwrap()
                        .get_object(&"img_image")
                        .expect("Could not get img_image?!");
                    let btn_num: gtk::SpinButton = arc_hdr_bar.lock().unwrap().get_titlebar_button("btn_num").unwrap();
                    //img_image.set_from_file(fc.get_filename().unwrap());

                    match glitch_imagefile_by_numbytes(&std::path::PathBuf::from(&*arc_filename.lock().unwrap()), btn_num.get_value_as_int() as u32) {
                        Ok(buf) =>  {
                            img_image.set_from_pixbuf(Some(&buf));
                            // Resize ApplicationWindow to fit screen (todo: use actual screen dimensions)
                            let img_alloc = img_image.get_allocation();
                            let w = img_alloc.width.max(1920);
                            let h = img_alloc.height.max(1080);
                            arc_builder.lock().unwrap().get_object::<gtk::ApplicationWindow>("MainWindow").unwrap().
                                set_size_request(w, h);
                            img_image.set_size_request(w, h);
                        },
                        Err(_) => {
                            // Show error
                            let dlg = gtk::MessageDialog::new(
                                Some(&((arc_builder.lock().unwrap()).get_object::<gtk::ApplicationWindow>("MainWindow").unwrap())),
                                gtk::DialogFlags::empty(), 
                                gtk::MessageType::Error,
                                gtk::ButtonsType::Ok,
                                &"Could not parse the glitched image, it might be glitched too strongly!"
                            );
                            
                            dlg.run();
                            dlg.destroy();
                        }
                    }
                };
                fc.destroy();
            });
        }

        { 
            let arc_builder = arc_builder.clone();
            let arc_filename = arc_filename.clone();

            let btn_num: gtk::SpinButton = arc_hdr_bar.lock().unwrap().get_titlebar_button("btn_num").expect("Could not get btn_num?!");
            btn_num.connect_changed(move |me| {
                let value = me.get_value_as_int();
                let input_file_name = &*arc_filename.lock().unwrap(); // what is this &* and why does it work lmao
                if input_file_name.is_empty() { return; }

                let img_image: gtk::Image = arc_builder.lock().unwrap().get_object(&"img_image").expect("Could not get img_image?!");

                match glitch_imagefile_by_numbytes(&std::path::PathBuf::from(input_file_name), value as u32) {
                    Ok(buf) => img_image.set_from_pixbuf(Some(&buf)),
                    Err(_) => {
                        // Show error
                        let dlg = gtk::MessageDialog::new(
                            Some(&((arc_builder.lock().unwrap()).get_object::<gtk::ApplicationWindow>("MainWindow").unwrap())),
                            gtk::DialogFlags::empty(), 
                            gtk::MessageType::Error,
                            gtk::ButtonsType::Ok,
                            &"Could not parse the glitched image, it might be glitched too strongly!"
                        );
                        
                        dlg.run();
                        dlg.destroy();
                    }
                }
            });
        }

        {
            let arc_hdr_bar = arc_hdr_bar.clone();
            let arc_filename = arc_filename.clone();

            let btn_save: gtk::Button = arc_hdr_bar.lock().unwrap().get_titlebar_button("btn_save").expect("Could not get btn_save?!");
            btn_save.connect_clicked(move |_| {
                if arc_filename.lock().unwrap().is_empty() { return; }

                let fcb = gtk::FileChooserDialogBuilder::new();
                let fc: FileChooserDialog = fcb.build();
                fc.set_action(gtk::FileChooserAction::Save);
                fc.add_buttons(&[
                    (&"Save", gtk::ResponseType::Ok),
                    (&"Cancel", gtk::ResponseType::Cancel),
                ]);

                let file_filter = FileFilter::new();
                file_filter.set_name(Some("JPEG images (*.jpg)"));
                file_filter.add_pattern(&"*.jpg");
                file_filter.add_pattern(&"*.jpeg");
                fc.add_filter(&file_filter);

                let file_filter = FileFilter::new();
                file_filter.set_name(Some("PNG images (*.png)"));
                file_filter.add_pattern(&"*.png");
                fc.add_filter(&file_filter);

                if let gtk::ResponseType::Ok = fc.run() {
                    // ok, load file
                    let save_filename = fc.get_filename().unwrap().to_str().unwrap().to_string();
                    println!("Saving pixbuf to: {}", save_filename);

                    let img_image: gtk::Image = arc_builder.lock().unwrap()
                        .get_object(&"img_image")
                        .expect("Could not get img_image?!");
                    
                    match save_pixbuf(&img_image.get_pixbuf().unwrap(), &fc.get_filename().unwrap()) {
                        Ok(_) => {
                            // file saved successfully
                        },
                        Err(e) => {
                            // Error saving file, display error box!
                            println!("Save failed: {:?}", e);

                            let dlg = gtk::MessageDialog::new(
                                Some(&((arc_builder.lock().unwrap()).get_object::<gtk::ApplicationWindow>("MainWindow").unwrap())),
                                gtk::DialogFlags::empty(), 
                                gtk::MessageType::Error,
                                gtk::ButtonsType::Ok,
                                &e.to_string()
                            );
                            
                            dlg.run();
                            dlg.destroy();
                        }
                    }
                };
                fc.destroy();
            });
        }
        

        main_window.show_all();
    });

    uiapp.run(&env::args().collect::<Vec<_>>());
}
