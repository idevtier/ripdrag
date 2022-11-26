use std::io::{self, BufRead};
use std::path::PathBuf;
use std::str::FromStr;
use std::thread;

use crate::cli::Cli;
use clap::Parser;
use gtk::gdk::DragAction;
use gtk::glib::{self, clone, Bytes, MainContext, Type, PRIORITY_DEFAULT};
use gtk::{
    gdk::ContentProvider, prelude::*, Application, ApplicationWindow, Button, DragSource,
    DropTarget, EventControllerKey, ListBox, PolicyType, ScrolledWindow,
};
use gtk::{CenterBox, Image, Orientation};
use url::Url;

pub fn build_ui(app: &Application) {
    // Parse arguments and check if files exist
    let args = Cli::parse();
    for path in &args.paths {
        assert!(
            path.exists(),
            "{0} : no such file or directory",
            path.display()
        );
    }
    // Create a scrollable list
    let list_box = ListBox::new();
    let scrolled_window = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never) //  Disable horizontal scrolling
        .min_content_width(args.content_width)
        .child(&list_box)
        .build();

    // Build the main window
    let window = ApplicationWindow::builder()
        .title("ri pdrag")
        .resizable(args.resizable)
        .application(app)
        .child(&scrolled_window)
        .default_height(args.content_height)
        .build();

    if args.target {
        build_target_ui(list_box, args);
    } else {
        build_source_ui(list_box, args);
    }

    // Kill the app when Escape is pressed
    let event_controller = EventControllerKey::new();
    event_controller.connect_key_pressed(|_, key, _, _| {
        if key.name().unwrap() == "Escape" {
            std::process::exit(0)
        }
        return gtk::glib::signal::Inhibit(false);
    });

    window.add_controller(&event_controller);
    window.show();
}

fn build_source_ui(list_box: ListBox, args: Cli) {
    // Populate the list with the buttons, if there are any
    if args.paths.len() > 0 {
        if args.all_compact {
            list_box.append(&generate_compact(args.paths.clone(), args.and_exit));
        } else {
            for button in generate_buttons_from_paths(
                args.paths.clone(),
                args.and_exit,
                args.icons_only,
                args.disable_thumbnails,
                args.icon_size,
                args.all,
            ) {
                list_box.append(&button);
            }
        }
    }

    // Read from stdin and populate the list
    if args.from_stdin {
        let mut paths: Vec<PathBuf> = args.paths.clone();
        let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);
        thread::spawn(move || {
            let stdin = io::stdin();
            let mut lines = stdin.lock().lines();

            while let Some(line) = lines.next() {
                let path = PathBuf::from(line.unwrap());
                if path.exists() {
                    println!("Adding: {}", path.display());
                    sender.send(path).expect("Error");
                } else {
                    if args.verbose {
                        println!("{} : no such file or directory", path.display())
                    }
                }
            }
        });
        receiver.attach(
                None,
                clone!(@weak list_box => @default-return Continue(false),
                            move |path| {
                                if args.all_compact{
                                    paths.push(path);
                                    match list_box.first_child(){
                                        Some(child) => list_box.remove(&child),
                                        None => {}
                                    };
                                    list_box.append(&generate_compact(paths.clone(),args.and_exit));
                                } else {
                                    let button = generate_buttons_from_paths(vec![path],args.and_exit, args.icons_only, args.disable_thumbnails, args.icon_size, args.all);
                                    list_box.append(&button[0]);
                                }
                                Continue(true)
                            }
                )
            );
    }
}

fn build_target_ui(list_box: ListBox, args: Cli) {
    // Generate the Drop Target and button
    let button = Button::builder().label("Drop your files here").build();

    let drop_target = DropTarget::new(Type::INVALID, DragAction::COPY);
    // TODO: This is borken on anything other than linux
    // Figure out a way to accept G_TYPE_FILE other than STRING
    drop_target.set_types(&vec![Type::STRING]);

    let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);

    drop_target.connect_drop(move |_, value, _, _| {
        match value.get() {
            Ok(val) => {
                // safely extract the path string from the value
                let data: String = val;
                if args.print_path {
                    data.lines().for_each(|line| match Url::from_str(&line) {
                        Ok(url) => match url.to_file_path() {
                            Ok(path) => {
                                println!("{}", path.canonicalize().unwrap().to_string_lossy())
                            }
                            Err(_) => {
                                if args.verbose {
                                    println!("Cannot convert path to string")
                                }
                            }
                        },
                        Err(_) => {
                            if args.verbose {
                                println!("Cannot convert drop data to url")
                            }
                        }
                    });
                } else {
                    data.lines().for_each(|line| println!("{}", line));
                }
                if args.keep {
                    sender.send(data).expect("Error");
                }
            }
            Err(_) => {
                if args.verbose {
                    println!("Cannot decode drop data")
                }
            }
        };
        true
    });

    // get the uri_list from the drop and populate the list of files (--keep)
    let mut paths: Vec<PathBuf> = Vec::new();
    receiver.attach(
        None,
        clone!(@weak list_box => @default-return Continue(false),
                move |uri_list| {
                    let mut new_paths :Vec<PathBuf> = Vec::new();
                    uri_list.lines().for_each(|uri| {
                            match Url::from_str(uri) {
                                Ok(url) => {match url.to_file_path() {
                                    Ok(path) => {new_paths.push(path)},
                                    Err(_) => {}
                                }},
                                Err(_) => {}
                            };
                    });
                    if args.all_compact{
                        // Hacky solution, check if we already created buttons
                        if !paths.is_empty(){
                            match list_box.last_child(){
                                Some(child) => {
                                    list_box.remove(&child)
                                },
                                None => {}
                            };
                        }
                        paths.append(&mut new_paths);
                        list_box.append(&generate_compact(paths.clone(),args.and_exit));
                    } else {
                        // This solution is fast, but it's gonna cause problems when --all is used in combinatio with --target
                        for button in &generate_buttons_from_paths(new_paths, args.and_exit, args.icons_only, args.disable_thumbnails, args.icon_size, args.all){
                            list_box.append(button);           
                        };
                    }
                    Continue(true)
                }
        )
    );
    button.add_controller(&drop_target);
    list_box.append(&button);
}

fn generate_compact(paths: Vec<PathBuf>, and_exit: bool) -> Button {
    // Here we want to generate a single draggable button, containg all the files
    let button = Button::builder()
        .label(&format!("{} elements", paths.len()))
        .build();
    let drag_source = DragSource::builder().build();

    drag_source.connect_prepare(move |_, _, _| {
        Some(ContentProvider::for_bytes(
            "text/uri-list",
            &generate_uri_list(&paths),
        ))
    });

    if and_exit {
        drag_source.connect_drag_end(|_, _, _| std::process::exit(0));
    }
    button.add_controller(&drag_source);
    return button;
}

fn generate_uri_list(paths: &Vec<PathBuf>) -> Bytes {
    return gtk::glib::Bytes::from_owned(
        paths
            .iter()
            .map(|path| -> String {
                Url::from_file_path(path.canonicalize().unwrap())
                    .unwrap()
                    .to_string()
            })
            .reduce(|accum, item| [accum, item].join("\n"))
            .unwrap(),
    );
}

fn generate_buttons_from_paths(
    paths: Vec<PathBuf>,
    and_exit: bool,
    icons_only: bool,
    disable_thumbnails: bool,
    icon_size: i32,
    all: bool,
) -> Vec<Button> {
    let mut button_vec = Vec::new();
    let uri_list = generate_uri_list(&paths);

    //TODO: make this loop multithreaded
    for path in paths.into_iter() {
        // The CenterBox(button_box) contains the image and the optional label
        // The Button contains the CenterBox and can be dragged
        let button_box = CenterBox::builder()
            .orientation(Orientation::Horizontal)
            .build();

        match get_image_from_path(&path, icon_size, disable_thumbnails) {
            Some(image) => {
                if icons_only {
                    button_box.set_center_widget(Some(&image));
                } else {
                    button_box.set_start_widget(Some(&image));
                }
            }
            None => {}
        };

        if !icons_only {
            button_box.set_center_widget(Some(
                &gtk::Label::builder()
                    .label(path.display().to_string().as_str())
                    .build(),
            ));
        }

        let button = Button::builder().child(&button_box).build();
        let drag_source = DragSource::builder().build();

        if all {
            let list = uri_list.clone();
            drag_source.connect_prepare(move |_, _, _| {
                Some(ContentProvider::for_bytes("text/uri-list", &list))
            });
        } else {
            let uri = generate_uri_from_path(&path);
            drag_source.connect_prepare(move |_, _, _| {
                Some(ContentProvider::for_bytes("text/uri-list", &uri))
            });
        }

        if and_exit {
            drag_source.connect_drag_end(|_, _, _| std::process::exit(0));
        }

        // Open the path with the default app
        button.connect_clicked(move |_| {
            opener::open(&path).unwrap();
        });

        button.add_controller(&drag_source);
        button_vec.push(button);
    }
    return button_vec;
}

fn get_image_from_path(
    path: &std::path::PathBuf,
    icon_size: i32,
    disable_thumbnails: bool,
) -> Option<Image> {
    let mime_type;
    if path.metadata().unwrap().is_dir() {
        mime_type = "inode/directory";
    } else {
        mime_type = match infer::get_from_path(&path) {
            Ok(option) => match option {
                Some(infer_type) => infer_type.mime_type(),
                None => "text/plain",
            },
            Err(_) => "text/plain",
        };
    }
    if mime_type.contains("image") & !disable_thumbnails {
        return Some(
            Image::builder()
                .file(&path.as_os_str().to_str().unwrap())
                .pixel_size(icon_size)
                .build(),
        );
    }
    return match gtk::gio::content_type_get_generic_icon_name(mime_type) {
        Some(icon_name) => Some(
            Image::builder()
                .icon_name(&icon_name)
                .pixel_size(icon_size)
                .build(),
        ),
        None => None,
    };
}

fn generate_uri_from_path(path: &PathBuf) -> Bytes {
    return gtk::glib::Bytes::from_owned(
        Url::from_file_path(path.canonicalize().unwrap())
            .unwrap()
            .to_string(),
    );
}
