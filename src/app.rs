//! Application model

mod components;

use adw::prelude::{AdwApplicationWindowExt, GtkWindowExt};
use anyhow::Result;
use gtk::prelude::{SettingsExt, WidgetExt};
use gtk::{gdk, gio};
use relm4::actions::{AccelsPlus, RelmAction, RelmActionGroup};
use relm4::{AppUpdate, RelmApp, RelmComponent, Sender};

use super::config::{APP_ID, PKGDATADIR, PROFILE, VERSION};
use components::{about_dialog, help_overlay, leaflet};

/// Model
struct Model {
    /// Settings
    settings: gio::Settings,
}

/// Settings
struct Settings {
    /// Width of the application window
    width: i32,
    /// Height of the application window
    height: i32,
    /// Is the application window maximized?
    is_maximized: bool,
}

impl Model {
    /// Save the settings
    fn save_settings(&self, settings: &Settings) -> Result<()> {
        self.settings.set_int("window-width", settings.width)?;
        self.settings.set_int("window-height", settings.height)?;
        self.settings
            .set_boolean("is-maximized", settings.is_maximized)?;
        Ok(())
    }
}

/// Messages
enum Msg {
    /// Close the application, saving the settings
    Close(Settings),
}

/// Components
#[derive(relm4_macros::Components)]
struct Components {
    /// About Dialog
    about_dialog: RelmComponent<about_dialog::Model, Model>,
    /// Help Overlay
    help_overlay: RelmComponent<help_overlay::Model, Model>,
    /// Leaflet
    leaflet: RelmComponent<leaflet::Model, Model>,
}

impl relm4::Model for Model {
    type Msg = Msg;
    type Widgets = Widgets;
    type Components = Components;
}

impl AppUpdate for Model {
    fn update(&mut self, msg: Msg, _components: &Components, _sender: Sender<Msg>) -> bool {
        match msg {
            Msg::Close(settings) => self.save_settings(&settings).is_ok(),
        }
    }
}

#[allow(clippy::missing_docs_in_private_items)]
#[relm4_macros::widget]
impl relm4::Widgets<Model, ()> for Widgets {
    view! {
        // Application Window
        app_window = adw::ApplicationWindow {
            set_default_width: model.settings.int("window-width"),
            set_default_height: model.settings.int("window-height"),
            set_maximized: model.settings.boolean("is-maximized"),
            connect_close_request(sender) => move |w| {
                // Prepare settings
                let settings = Settings {
                    width: w.default_width(),
                    height: w.default_height(),
                    is_maximized: w.is_maximized(),
                };
                // Prepare a message
                let msg = Msg::Close(settings);
                // Send the message
                sender.send(msg).ok();
                gtk::Inhibit(false)
            },
            // Leaflet
            set_content: Some(components.leaflet.root_widget())
        }
    }
    fn post_init() {
        // Prepare action groups
        let window_actions = RelmActionGroup::<WindowActionGroup>::new();
        let application_actions = RelmActionGroup::<ApplicationActionGroup>::new();
        // Create the Open Help Overlay action
        let open_help_overlay_action: RelmAction<OpenHelpOverlay> = RelmAction::new_stateless({
            let sender = components.help_overlay.sender();
            move |_| {
                sender.send(help_overlay::Msg::Open).ok();
            }
        });
        // Create the Open About Dialog action
        let open_about_dialog_action: RelmAction<OpenAboutDialog> = RelmAction::new_stateless({
            let sender = components.about_dialog.sender();
            move |_| {
                sender.send(about_dialog::Msg::Open).ok();
            }
        });
        // Create the Close Application action
        let close_application_action: RelmAction<CloseApplication> = RelmAction::new_stateless({
            let w = app_window.clone();
            move |_| {
                w.close();
            }
        });
        // Add the actions to the according group
        window_actions.add_action(open_help_overlay_action);
        application_actions.add_action(open_about_dialog_action);
        application_actions.add_action(close_application_action);
        // Insert the action groups into the window
        let window_actions_group = window_actions.into_action_group();
        let application_actions_group = application_actions.into_action_group();
        app_window.insert_action_group("win", Some(&window_actions_group));
        app_window.insert_action_group("app", Some(&application_actions_group));
        // Set accelerators for the actions
        let app = relm4::gtk_application();
        app.set_accelerators_for_action::<CloseApplication>(&["<primary>Q"]);
        app.set_accelerators_for_action::<OpenHelpOverlay>(&["<primary>question"]);
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
    // Log the application info
    log::info!("Tidings ({})", APP_ID);
    log::info!("Version: {} ({})", VERSION, PROFILE);
    log::info!("Datadir: {}", PKGDATADIR);
    // Run the application
    app.run();
}

relm4::new_action_group!(WindowActionGroup, "win");
relm4::new_stateless_action!(OpenHelpOverlay, WindowActionGroup, "show-help-overlay");

relm4::new_action_group!(ApplicationActionGroup, "app");
relm4::new_stateless_action!(OpenAboutDialog, ApplicationActionGroup, "about");
relm4::new_stateless_action!(CloseApplication, ApplicationActionGroup, "quit");
