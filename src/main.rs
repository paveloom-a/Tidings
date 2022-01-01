mod application;
mod config;

use anyhow::{Context, Result};
use gettextrs::{gettext, LocaleCategory};
use gtk::glib;

use config::{DOMAINNAME, LOCALEDIR};

fn main() -> Result<()> {
    // Initialize the logger
    pretty_env_logger::init();

    // Prepare i18n
    gettextrs::setlocale(LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(DOMAINNAME, LOCALEDIR)
        .with_context(|| "Unable to bind the text domain")?;
    gettextrs::textdomain(DOMAINNAME).with_context(|| "Unable to switch to the text domain")?;

    glib::set_application_name(&gettext("Tidings"));

    application::run();
    Ok(())
}
