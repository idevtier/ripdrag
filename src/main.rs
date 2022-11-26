use gtk::gio::ApplicationFlags;
use gtk::glib::set_program_name;
use gtk::{prelude::*, Application};

mod cli;
mod ui;

fn main() {
    set_program_name(Some("ripdrag"));
    let app = Application::builder()
        .application_id("ga.strin.ripdrag")
        .flags(ApplicationFlags::NON_UNIQUE)
        .build();
    app.connect_activate(ui::build_ui);
    app.run_with_args(&vec![""]); // we don't want gtk to parse the arguments. cleaner solutions are welcome
}
