//! Application model

mod actions;
mod components;

use adw::prelude::{AdwApplicationWindowExt, GtkWindowExt};
use gtk::prelude::{SettingsExt, WidgetExt};
use gtk::{gdk, gio};
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, MessageBroker,
    RelmApp, SimpleComponent,
};

use std::convert::identity;

use super::config::{APP_ID, PROFILE};
use actions::{setup_accels, setup_actions};
use components::{about_dialog, add_directory_dialog, add_feed_dialog, help_overlay, leaflet};

/// Message broker
pub static BROKER: MessageBroker<Model> = MessageBroker::new();

/// Model
pub struct Model {
    /// Settings
    settings: gio::Settings,
    /// Is the application running?
    running: bool,
    /// About Dialog
    #[allow(dead_code)]
    about_dialog: Controller<about_dialog::Model>,
    /// Help Overlay
    #[allow(dead_code)]
    help_overlay: Controller<help_overlay::Model>,
    /// Add Feed Dialog
    #[allow(dead_code)]
    add_feed_dialog: Controller<add_feed_dialog::Model>,
    /// Add Directory Dialog
    #[allow(dead_code)]
    add_directory_dialog: Controller<add_directory_dialog::Model>,
    /// Leaflet
    leaflet: Controller<leaflet::Model>,
}

/// Settings
#[derive(Debug, Copy, Clone)]
pub struct Settings {
    /// Width of the application window
    width: i32,
    /// Height of the application window
    height: i32,
    /// Is the application window maximized?
    is_maximized: bool,
}

impl From<&adw::ApplicationWindow> for Settings {
    fn from(w: &adw::ApplicationWindow) -> Self {
        Self {
            width: w.default_width(),
            height: w.default_height(),
            is_maximized: w.is_maximized(),
        }
    }
}

impl Model {
    /// Save the settings
    fn save_settings(&self, settings: &Settings) {
        self.settings.set_int("window-width", settings.width).ok();
        self.settings.set_int("window-height", settings.height).ok();
        self.settings
            .set_boolean("is-maximized", settings.is_maximized)
            .ok();
    }
}

/// Messages
#[derive(Debug)]
pub enum Msg {
    /// Save the settings
    Save(Settings),
    /// Quit the application
    Quit,
}

#[allow(clippy::clone_on_ref_ptr)]
#[allow(clippy::missing_docs_in_private_items)]
#[allow(unused_variables)]
#[relm4::component(pub)]
impl SimpleComponent for Model {
    type Init = ();
    type Input = Msg;
    type Output = ();
    type Widgets = Widgets;
    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // Initialize the components
        let leaflet = leaflet::Model::builder()
            .launch_with_broker((), &leaflet::BROKER)
            .forward(sender.input_sender(), identity);
        let about_dialog = about_dialog::Model::builder()
            .launch_with_broker((), &about_dialog::BROKER)
            .forward(sender.input_sender(), identity);
        let help_overlay = help_overlay::Model::builder()
            .launch_with_broker((), &help_overlay::BROKER)
            .forward(sender.input_sender(), identity);
        let add_feed_dialog = add_feed_dialog::Model::builder()
            .launch_with_broker((), &add_feed_dialog::BROKER)
            .forward(sender.input_sender(), identity);
        let add_directory_dialog = add_directory_dialog::Model::builder()
            .launch_with_broker((), &add_directory_dialog::BROKER)
            .forward(sender.input_sender(), identity);
        // Initialize the model
        let model = Self {
            settings: gio::Settings::new(APP_ID),
            running: true,
            leaflet,
            about_dialog,
            help_overlay,
            add_feed_dialog,
            add_directory_dialog,
        };
        // Set the components as transient to the root
        model.about_dialog.widget().set_transient_for(Some(root));
        model.help_overlay.widget().set_transient_for(Some(root));
        model.add_feed_dialog.widget().set_transient_for(Some(root));
        model
            .add_directory_dialog
            .widget()
            .set_transient_for(Some(root));
        let widgets = view_output!();
        // Setup actions
        setup_actions(&widgets.app_window);
        // Add a CSS style to the window if it's a development build
        if PROFILE == "dev" {
            widgets.app_window.add_css_class("devel");
        }
        ComponentParts { model, widgets }
    }
    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            Msg::Save(settings) => {
                // Save the settings
                self.save_settings(&settings);
            }
            Msg::Quit => {
                // Quit the application
                self.running = false;
            }
        }
    }
    fn pre_view() {
        // In case of a quit request from an
        // accelerator, quit the application
        if !model.running {
            // Close the application window
            app_window.close();
        }
    }
    view! {
        // Application Window
        app_window = adw::ApplicationWindow {
            set_default_width: model.settings.int("window-width"),
            set_default_height: model.settings.int("window-height"),
            set_maximized: model.settings.boolean("is-maximized"),
            connect_close_request[sender] => move |w| {
                // Prepare settings
                let settings = Settings::from(w);
                // Save the settings
                sender.input(Msg::Save(settings));
                gtk::Inhibit(false)
            },
            add_controller = &gtk::EventControllerKey {
                connect_key_pressed[sender] => move |_, key, _, _| {
                    // Esc: Return to the feeds
                    if key == gdk::Key::Escape {
                        leaflet::BROKER.send(leaflet::Msg::HideTidingsPage);
                    }
                    gtk::Inhibit(false)
                }
            },
            // Leaflet
            set_content: Some(model.leaflet.widget())
        }
    }
}

/// Run the application
pub fn run() {
    // Initialize the application
    let app = RelmApp::new(APP_ID);
    // Set the accelerators for the actions
    setup_accels(&app.app);
    // Add a CSS provider
    let provider = gtk::CssProvider::new();
    provider.load_from_resource("/paveloom/apps/tidings/style.css");
    if let Some(display) = gdk::Display::default() {
        gtk::StyleContext::add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
    // Run the application
    app.run::<Model>(());
}
