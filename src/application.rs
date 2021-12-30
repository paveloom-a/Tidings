use anyhow::{Context, Result};
use gettextrs::gettext;
use log::info;

use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib};

use crate::config::{APP_ID, PKGDATADIR, PROFILE, VERSION};
use crate::window::ExampleApplicationWindow;

mod imp {
    use gtk::glib::{self, WeakRef};
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use log::debug;
    use once_cell::sync::OnceCell;

    use crate::config::APP_ID;
    use crate::window::ExampleApplicationWindow;

    #[derive(Debug, Default)]
    pub struct ExampleApplication {
        pub window: OnceCell<WeakRef<ExampleApplicationWindow>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ExampleApplication {
        const NAME: &'static str = "ExampleApplication";
        type Type = super::ExampleApplication;
        type ParentType = gtk::Application;
    }

    impl ObjectImpl for ExampleApplication {}

    impl ApplicationImpl for ExampleApplication {
        fn activate(&self, app: &Self::Type) {
            debug!("GtkApplication<ExampleApplication>::activate");

            if let Some(window) = self.window.get() {
                let window = window.upgrade().unwrap();
                window.show();
                window.present();
                return;
            }

            let window = ExampleApplicationWindow::new(app);
            self.window
                .set(window.downgrade())
                .expect("Window already set.");

            app.main_window().present();
        }

        fn startup(&self, app: &Self::Type) {
            debug!("GtkApplication<ExampleApplication>::startup");
            self.parent_startup(app);

            // Set icons for shell
            gtk::Window::set_default_icon_name(APP_ID);

            crate::application::ExampleApplication::setup_css();
            app.setup_gactions();
            app.setup_accels();
        }
    }

    impl GtkApplicationImpl for ExampleApplication {}
}

glib::wrapper! {
    pub struct ExampleApplication(ObjectSubclass<imp::ExampleApplication>)
        @extends gio::Application, gtk::Application,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl ExampleApplication {
    pub fn new() -> Result<Self> {
        glib::Object::new(&[
            ("application-id", &Some(APP_ID)),
            ("flags", &gio::ApplicationFlags::empty()),
            ("resource-base-path", &Some("/paveloom/tidings/")),
        ])
        .with_context(|| "Application initialization failed...")
    }

    fn main_window(&self) -> ExampleApplicationWindow {
        let imp = imp::ExampleApplication::from_instance(self);
        imp.window.get().unwrap().upgrade().unwrap()
    }

    fn setup_gactions(&self) {
        // Quit
        let action_quit = gio::SimpleAction::new("quit", None);
        action_quit.connect_activate(clone!(@weak self as app => move |_, _| {
            // This is needed to trigger the delete event and saving the window state
            app.main_window().close();
            app.quit();
        }));
        self.add_action(&action_quit);

        // About
        let action_about = gio::SimpleAction::new("about", None);
        action_about.connect_activate(clone!(@weak self as app => move |_, _| {
            app.show_about_dialog();
        }));
        self.add_action(&action_about);
    }

    // Sets up keyboard shortcuts
    fn setup_accels(&self) {
        self.set_accels_for_action("app.quit", &["<primary>q"]);
    }

    fn setup_css() {
        let provider = gtk::CssProvider::new();
        provider.load_from_resource("/paveloom/tidings/style.css");
        if let Some(display) = gdk::Display::default() {
            gtk::StyleContext::add_provider_for_display(
                &display,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    }

    fn show_about_dialog(&self) {
        let dialog = gtk::AboutDialogBuilder::new()
            .logo_icon_name(APP_ID)
            // Insert your license of choice here
            .license_type(gtk::License::Gpl30Only)
            // Insert your website here
            .website("https://github.com/paveloom-a/Tidings")
            .version(VERSION)
            .transient_for(&self.main_window())
            .translator_credits(&gettext("translator-credits"))
            .modal(true)
            .authors(vec!["Pavel Sobolev".into()])
            .artists(vec!["Pavel Sobolev".into()])
            .build();

        dialog.show();
    }

    pub fn run(&self) {
        info!("Tidings ({})", APP_ID);
        info!("Version: {} ({})", VERSION, PROFILE);
        info!("Datadir: {}", PKGDATADIR);

        ApplicationExtManual::run(self);
    }
}