//! This binary crate is the main executable of the application

mod app;
mod config;

use anyhow::{Context, Result};
use gettextrs::{gettext, LocaleCategory};
use gtk::{gio, glib};

use config::{DOMAINNAME, LOCALEDIR, RESOURCES_FILE};

fn main() -> Result<()> {
    // Initialize the logger
    pretty_env_logger::init();
    // Prepare i18n
    gettextrs::setlocale(LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(DOMAINNAME, LOCALEDIR)
        .with_context(|| "Unable to bind the text domain")?;
    gettextrs::textdomain(DOMAINNAME).with_context(|| "Unable to switch to the text domain")?;
    // Set the application name
    glib::set_application_name(&gettext("Tidings"));
    // Load and register the resource bundle
    let res =
        gio::Resource::load(RESOURCES_FILE).with_context(|| "Could not load gresource file")?;
    gio::resources_register(&res);
    // Run the application
    app::run();
    Ok(())
}
