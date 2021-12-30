mod application;
mod config;
mod window;

use anyhow::{Context, Result};
use gettextrs::{gettext, LocaleCategory};
use gtk::{gio, glib};

use application::ExampleApplication;
use config::{DOMAINNAME, LOCALEDIR, RESOURCES_FILE};

fn main() -> Result<()> {
    // Initialize logger
    pretty_env_logger::init();

    // Prepare i18n
    gettextrs::setlocale(LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(DOMAINNAME, LOCALEDIR)
        .with_context(|| "Unable to bind the text domain")?;
    gettextrs::textdomain(DOMAINNAME).with_context(|| "Unable to switch to the text domain")?;

    glib::set_application_name(&gettext("Tidings"));

    gtk::init().with_context(|| "Unable to initialize the windowing system")?;

    // Load and register the resource bundle
    // (See https://docs.gtk.org/gio/struct.Resource.html)
    let res =
        gio::Resource::load(RESOURCES_FILE).with_context(|| "Could not load gresource file")?;
    gio::resources_register(&res);

    let app = ExampleApplication::new()?;
    app.run();
    Ok(())
}
