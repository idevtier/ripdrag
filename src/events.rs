use gtk::EventControllerKey;

pub fn exit_on_escape_key_pressed() -> EventControllerKey {
    let event_controller = EventControllerKey::new();
    event_controller.connect_key_pressed(|_, key, _, _| {
        if key.name().unwrap() == "Escape" {
            std::process::exit(0)
        }
        return gtk::glib::signal::Inhibit(false);
    });
    event_controller
}
