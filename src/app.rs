//! Application model

mod components;

use adw::prelude::{AdwApplicationWindowExt, GtkWindowExt};
use anyhow::{Context, Result};
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

impl Model {
    /// Save the settings
    fn save_settings(&self, width: i32, height: i32, is_maximized: bool) -> Result<()> {
        self.settings
            .set_int("window-width", width)
            .with_context(|| "Couldn't save the width of the window")?;
        self.settings
            .set_int("window-height", height)
            .with_context(|| "Couldn't save the height of the window")?;
        self.settings
            .set_boolean("is-maximized", is_maximized)
            .with_context(|| "Couldn't save the `is-maximized` property of the window")?;
        Ok(())
    }
}

/// Messages
enum Msg {
    /// Open About Dialog
    OpenAboutDialog,
    /// Open Help Overlay
    OpenHelpOverlay,
    /// Close the application, saving the settings
    Close(i32, i32, bool),
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
    fn update(&mut self, msg: Msg, components: &Components, _sender: Sender<Msg>) -> bool {
        match msg {
            Msg::OpenAboutDialog => components
                .about_dialog
                .send(about_dialog::Msg::Open)
                .is_ok(),
            Msg::OpenHelpOverlay => components
                .help_overlay
                .send(help_overlay::Msg::Open)
                .is_ok(),
            Msg::Close(width, height, is_maximized) => {
                self.save_settings(width, height, is_maximized).is_ok()
            }
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
                let msg = Msg::Close(w.default_width(), w.default_height(), w.is_maximized());
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
            let sender = sender.clone();
            move |_| {
                sender.send(Msg::OpenHelpOverlay).ok();
            }
        });
        // Create the Open About Dialog action
        let open_about_dialog_action: RelmAction<OpenAboutDialog> = RelmAction::new_stateless({
            let sender = sender.clone();
            move |_| {
                sender.send(Msg::OpenAboutDialog).ok();
            }
        });
        // Create the Close Application action
        let close_application_action: RelmAction<CloseApplication> = RelmAction::new_stateless({
            let app_window = app_window.clone();
            move |_| {
                // Prepare a message
                let msg = Msg::Close(
                    app_window.default_width(),
                    app_window.default_height(),
                    app_window.is_maximized(),
                );
                // Send it
                sender.send(msg).ok();
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
