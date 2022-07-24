//! Application model

mod actions;
mod components;

use adw::prelude::{AdwApplicationWindowExt, GtkWindowExt};
use gtk::prelude::{SettingsExt, WidgetExt};
use gtk::{gdk, gio};
use relm4::{AppUpdate, RelmApp, RelmComponent, Sender};

use super::config::{APP_ID, PROFILE};
use actions::setup_actions;
use components::{about_dialog, add_directory_dialog, add_feed_dialog, help_overlay, leaflet};

/// Model
struct Model {
    /// Settings
    settings: gio::Settings,
    /// Is the application running?
    running: bool,
}

/// Settings
#[derive(Copy, Clone)]
struct Settings {
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
enum Msg {
    /// Save the settings
    Save(Settings),
    /// Quit the application
    Quit,
    /// Transfer a message to the Feeds component
    TransferToFeeds(leaflet::feeds::Msg),
}

/// Components
#[derive(relm4::Components)]
struct Components {
    /// About Dialog
    about_dialog: RelmComponent<about_dialog::Model, Model>,
    /// Help Overlay
    help_overlay: RelmComponent<help_overlay::Model, Model>,
    /// Add Feed Dialog
    add_feed_dialog: RelmComponent<add_feed_dialog::Model, Model>,
    /// Add Directory Dialog
    add_directory_dialog: RelmComponent<add_directory_dialog::Model, Model>,
    /// Leaflet
    leaflet: RelmComponent<leaflet::Model, Model>,
}

impl relm4::Model for Model {
    type Msg = Msg;
    type Widgets = Widgets;
    type Components = Components;
}

impl AppUpdate for Model {
    fn update(&mut self, msg: Msg, components: &Components, _sender: Sender<Msg>) -> bool {
        match msg {
            Msg::Save(settings) => {
                // Save the settings
                self.save_settings(&settings);
            }
            Msg::Quit => {
                // Quit the application
                self.running = false;
            }
            Msg::TransferToFeeds(message) => {
                components
                    .leaflet
                    .send(leaflet::Msg::TransferToFeeds(message))
                    .ok();
            }
        }
        true
    }
}

#[allow(clippy::missing_docs_in_private_items)]
#[relm4::widget]
impl relm4::Widgets<Model, ()> for Widgets {
    view! {
        // Application Window
        app_window = adw::ApplicationWindow {
            set_default_width: model.settings.int("window-width"),
            set_default_height: model.settings.int("window-height"),
            set_maximized: model.settings.boolean("is-maximized"),
            connect_close_request(sender) => move |w| {
                // Prepare settings
                let settings = Settings::from(w);
                // Save the settings
                sender.send(Msg::Save(settings)).ok();
                gtk::Inhibit(false)
            },
            add_controller = &gtk::EventControllerKey {
                connect_key_pressed[
                    leaflet_sender = components.leaflet.sender(),
                ] => move |_, key, _, _| {
                    // Esc: Return to the feeds
                    if key == gdk::Key::Escape {
                        leaflet_sender.send(leaflet::Msg::HideTidingsPage).ok();
                    }
                    gtk::Inhibit(false)
                }
            },
            // Leaflet
            set_content: Some(components.leaflet.root_widget())
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
    fn post_init() {
        // Setup actions
        setup_actions(&app_window, components, &sender);
        // Add a CSS style to the window if it's a development build
        if PROFILE == "dev" {
            app_window.add_css_class("devel");
        }
    }
}

/// Run the application
pub fn run() {
    // Initialize the model
    let model = Model {
        settings: gio::Settings::new(APP_ID),
        running: true,
    };
    // Initialize the application
    let app = RelmApp::new(model);
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
    app.run();
}
