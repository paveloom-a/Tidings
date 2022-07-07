//! Application model

mod components;

use adw::prelude::GtkWindowExt;
use anyhow::{Context, Result};
use gtk::prelude::{SettingsExt, WidgetExt};
use gtk::{gdk, gio};
use relm4::actions::{AccelsPlus, RelmAction, RelmActionGroup};
use relm4::{AppUpdate, RelmApp, RelmComponent, Sender};

use super::config::{APP_ID, PKGDATADIR, PROFILE, VERSION};
use components::{about_dialog, feeds, feeds_back_button, help_overlay, tidings};

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
    /// Go back in the Feeds List View
    FeedsBack,
    /// Hide the back button in the Feeds List View
    FeedsHideBack,
    /// Show the back button in the Feeds List View
    FeedsShowBack,
    /// Close the application, saving the settings
    Close(i32, i32, bool),
}

/// Components
#[derive(relm4_macros::Components)]
struct AppComponents {
    /// About Dialog
    about_dialog: RelmComponent<about_dialog::Model, Model>,
    /// Help Overlay
    help_overlay: RelmComponent<help_overlay::Model, Model>,
    /// Feeds Back Button
    feeds_back_button: RelmComponent<feeds_back_button::Model, Model>,
    /// Feeds
    feeds: RelmComponent<feeds::Model, Model>,
    /// Tidings
    tidings: RelmComponent<tidings::Model, Model>,
}

impl relm4::Model for Model {
    type Msg = Msg;
    type Widgets = Widgets;
    type Components = AppComponents;
}

impl AppUpdate for Model {
    fn update(&mut self, msg: Msg, components: &AppComponents, _sender: Sender<Msg>) -> bool {
        match msg {
            Msg::OpenAboutDialog => match components.about_dialog.send(about_dialog::Msg::Open) {
                Ok(_) => true,
                Err(e) => {
                    log::error!("Couldn't send a message to open the About Dialog");
                    log::debug!("{e}");
                    false
                }
            },
            Msg::OpenHelpOverlay => match components.help_overlay.send(help_overlay::Msg::Open) {
                Ok(_) => true,
                Err(e) => {
                    log::error!("Couldn't send a message to open the Help Overlay");
                    log::debug!("{e}");
                    false
                }
            },
            Msg::FeedsBack => match components.feeds.send(feeds::Msg::Back) {
                Ok(_) => true,
                Err(e) => {
                    log::error!("Couldn't send a message to go back in the Feeds");
                    log::debug!("{e}");
                    false
                }
            },
            Msg::FeedsHideBack => {
                match components
                    .feeds_back_button
                    .send(feeds_back_button::Msg::Hide)
                {
                    Ok(_) => true,
                    Err(e) => {
                        log::error!(
                            "App: couldn't send a message to hide the Back button in the header"
                        );
                        log::debug!("{e}");
                        false
                    }
                }
            }
            Msg::FeedsShowBack => {
                match components
                    .feeds_back_button
                    .send(feeds_back_button::Msg::Show)
                {
                    Ok(_) => true,
                    Err(e) => {
                        log::error!(
                            "App: couldn't send a message to show the Back button in the header"
                        );
                        log::debug!("{e}");
                        false
                    }
                }
            }
            Msg::Close(width, height, is_maximized) => {
                self.save_settings(width, height, is_maximized)
                    .unwrap_or_else(|e| {
                        log::error!("Failed to save settings when closing Main Window");
                        log::debug!("{e}");
                    });
                false
            }
        }
    }
}

/// Initialize Main Window, filling it with the components
#[allow(clippy::expect_used)]
fn main_window(components: &AppComponents) -> adw::ApplicationWindow {
    // Initialize a GTK builder
    let builder = gtk::Builder::from_resource("/paveloom/apps/tidings/ui/main-window.ui");
    // Get the Main Window
    let main_window: adw::ApplicationWindow = builder
        .object("main_window")
        .expect("Failed to load Main Window UI");
    // Get the Feeds Header Bar
    let feeds_header_bar: adw::HeaderBar = builder
        .object("feeds_header_bar")
        .expect("Failed to load Feeds Header Bar UI");
    // Get the Feeds Scrolled Window
    let feeds_scrolled_window: gtk::ScrolledWindow = builder
        .object("feeds_scrolled_window")
        .expect("Failed to load Feeds Scrolled Window UI");
    // Get the Tidings Scrolled Window
    let tidings_scrolled_window: gtk::ScrolledWindow = builder
        .object("tidings_scrolled_window")
        .expect("Failed to load Tidings Scrolled Window UI");
    // Connect the components to their parents
    feeds_header_bar.pack_start(components.feeds_back_button.root_widget());
    feeds_scrolled_window.set_child(Some(components.feeds.root_widget()));
    tidings_scrolled_window.set_child(Some(components.tidings.root_widget()));
    // Return Main Window
    main_window
}

#[allow(clippy::missing_docs_in_private_items)]
#[relm4_macros::widget]
impl relm4::Widgets<Model, ()> for Widgets {
    view! {
        main_window = main_window(components) -> adw::ApplicationWindow {
            set_default_width: model.settings.int("window-width"),
            set_default_height: model.settings.int("window-height"),
            set_maximized: model.settings.boolean("is-maximized"),
            connect_close_request(sender) => move |w| {
                let msg = Msg::Close(w.default_width(), w.default_height(), w.is_maximized());
                sender.send(msg).unwrap_or_else(|e| {
                    log::error!("Couldn't send a message to close the Main Window");
                    log::debug!("{e}");
                });
                gtk::Inhibit(true)
            },
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
                sender.send(Msg::OpenHelpOverlay).unwrap_or_else(|e| {
                    log::error!("Main Window: couldn't send a message to open the Help Overlay");
                    log::debug!("{e}");
                });
            }
        });
        // Create the Open About Dialog action
        let open_about_dialog_action: RelmAction<OpenAboutDialog> = RelmAction::new_stateless({
            let sender = sender.clone();
            move |_| {
                sender.send(Msg::OpenAboutDialog).unwrap_or_else(|e| {
                    log::error!("Main Window: couldn't send a message to open the About Dialog");
                    log::debug!("{e}");
                });
            }
        });
        // Create the Close Application action
        let close_application_action: RelmAction<CloseApplication> = RelmAction::new_stateless({
            let main_window = main_window.clone();
            move |_| {
                // Prepare a message
                let msg = Msg::Close(
                    main_window.default_width(),
                    main_window.default_height(),
                    main_window.is_maximized(),
                );
                // Send it
                sender.send(msg).unwrap_or_else(|e| {
                    log::error!("Main Window: couldn't send a message to close the application");
                    log::debug!("{e}");
                });
            }
        });
        // Add the actions to the according group
        window_actions.add_action(open_help_overlay_action);
        application_actions.add_action(open_about_dialog_action);
        application_actions.add_action(close_application_action);
        // Insert the action groups into the window
        let application_actions_group = application_actions.into_action_group();
        let window_actions_group = window_actions.into_action_group();
        main_window.insert_action_group("win", Some(&window_actions_group));
        main_window.insert_action_group("app", Some(&application_actions_group));
        // Set accelerators for the actions
        let app = relm4::gtk_application();
        app.set_accelerators_for_action::<OpenHelpOverlay>(&["<primary>question"]);
        app.set_accelerators_for_action::<CloseApplication>(&["<primary>Q"]);
        // Add a CSS style to the window if it's a development build
        if PROFILE == "dev" {
            main_window.add_css_class("devel");
        }
    }
}

relm4::new_action_group!(WindowActionGroup, "win");
relm4::new_stateless_action!(OpenHelpOverlay, WindowActionGroup, "show-help-overlay");

relm4::new_action_group!(ApplicationActionGroup, "app");
relm4::new_stateless_action!(OpenAboutDialog, ApplicationActionGroup, "about");
relm4::new_stateless_action!(OpenPreferences, ApplicationActionGroup, "preferences");
relm4::new_stateless_action!(CloseApplication, ApplicationActionGroup, "quit");

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
