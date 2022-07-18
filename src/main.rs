//! This binary crate is the main executable of the application

mod app;
mod config;

use gettextrs::{gettext, LocaleCategory};
use gtk::{gio, glib};

use config::{DOMAINNAME, LOCALEDIR, RESOURCES_FILE};

fn main() {
    // Prepare i18n
    gettextrs::setlocale(LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(DOMAINNAME, LOCALEDIR).ok();
    gettextrs::textdomain(DOMAINNAME).ok();
    // Set the application name
    glib::set_application_name(&gettext("Tidings"));
    // Load and register the resource bundle
    if let Ok(bundle) = gio::Resource::load(RESOURCES_FILE) {
        gio::resources_register(&bundle);
    }
    // Run the application
    app::run();
}
