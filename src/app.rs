//! Application model

mod components;

use adw::prelude::GtkWindowExt;
use anyhow::Result;
use gtk::prelude::{SettingsExt, WidgetExt};
use gtk::{gdk, gio};
use log::{debug, info, warn};
use relm4::actions::{AccelsPlus, RelmAction, RelmActionGroup};
use relm4::{send, AppUpdate, RelmApp, RelmComponent, Sender};

use super::config::{APP_ID, PKGDATADIR, PROFILE, VERSION};
use components::{
    about_dialog, feeds_back_button, feeds_list_view, help_overlay, tidings_list_view,
};

/// Model
struct Model {
    /// Settings
    settings: gio::Settings,
}

impl Model {
    /// Set settings
    fn set_settings(&self, width: i32, height: i32, is_maximized: bool) -> Result<()> {
        self.settings.set_int("window-width", width)?;
        self.settings.set_int("window-height", height)?;
        self.settings.set_boolean("is-maximized", is_maximized)?;
        Ok(())
    }
}

/// Messages
enum Msg {
    /// Open About Dialog
    OpenAboutDialog,
    /// Open Help Overlay
    OpenHelpOverlay,
    /// Go back in the feeds list view
    FeedsBack,
    /// Hide the back button in the feeds list view
    FeedsHideBack,
    /// Show the back button in the feeds list view
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
    /// Feeds List View
    feeds_list_view: RelmComponent<feeds_list_view::Model, Model>,
    /// Tidings List View
    tidings_list_view: RelmComponent<tidings_list_view::Model, Model>,
}

impl relm4::Model for Model {
    type Msg = Msg;
    type Widgets = Widgets;
    type Components = AppComponents;
}

impl AppUpdate for Model {
    fn update(&mut self, msg: Msg, components: &AppComponents, _sender: Sender<Msg>) -> bool {
        match msg {
            Msg::OpenAboutDialog => {
                components
                    .about_dialog
                    .send(about_dialog::Msg::Open)
                    .unwrap();
                true
            }
            Msg::OpenHelpOverlay => {
                components
                    .help_overlay
                    .send(help_overlay::Msg::Open)
                    .unwrap();
                true
            }
            Msg::FeedsBack => {
                components
                    .feeds_list_view
                    .send(feeds_list_view::Msg::Back)
                    .unwrap();
                true
            }
            Msg::FeedsHideBack => {
                components
                    .feeds_back_button
                    .send(feeds_back_button::Msg::Hide)
                    .unwrap();
                true
            }
            Msg::FeedsShowBack => {
                components
                    .feeds_back_button
                    .send(feeds_back_button::Msg::Show)
                    .unwrap();
                true
            }
            Msg::Close(width, height, is_maximized) => {
                if let Err(err) = self.set_settings(width, height, is_maximized) {
                    warn!("Failed to set settings");
                    debug!("{}", &err);
                }
                false
            }
        }
    }
}

/// Get Main Window
fn main_window(components: &AppComponents) -> adw::ApplicationWindow {
    let builder = gtk::Builder::from_resource("/paveloom/apps/tidings/ui/main-window.ui");

    let main_window: adw::ApplicationWindow = builder
        .object("main_window")
        .expect("Failed to load Main Window UI");

    let feeds_header_bar: adw::HeaderBar = builder
        .object("feeds_header_bar")
        .expect("Failed to load Feeds Header Bar UI");

    let feeds_scrolled_window: gtk::ScrolledWindow = builder
        .object("feeds_scrolled_window")
        .expect("Failed to load Feeds Scrolled Window UI");

    let content_scrolled_window: gtk::ScrolledWindow = builder
        .object("content_scrolled_window")
        .expect("Failed to load Content Scrolled Window UI");

    feeds_header_bar.pack_start(components.feeds_back_button.root_widget());
    feeds_scrolled_window.set_child(Some(components.feeds_list_view.root_widget()));
    content_scrolled_window.set_child(Some(components.tidings_list_view.root_widget()));

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
                send!(sender, Msg::Close(w.default_width(), w.default_height(), w.is_maximized()));
                gtk::Inhibit(true)
            },
        }
    }

    fn post_init() {
        if PROFILE == "dev" {
            main_window.add_css_class("devel");
        }

        let window_actions = RelmActionGroup::<WindowActionGroup>::new();
        let application_actions = RelmActionGroup::<ApplicationActionGroup>::new();

        let sender_clone = sender.clone();
        let open_help_overlay_action: RelmAction<OpenHelpOverlay> =
            RelmAction::new_stateless(move |_| {
                send!(sender_clone, Msg::OpenHelpOverlay);
            });

        let sender_clone = sender.clone();
        let open_about_dialog_action: RelmAction<OpenAboutDialog> =
            RelmAction::new_stateless(move |_| {
                send!(sender_clone, Msg::OpenAboutDialog);
            });

        let main_window_clone = main_window.clone();
        let close_application_action: RelmAction<CloseApplication> =
            RelmAction::new_stateless(move |_| {
                send!(
                    sender,
                    Msg::Close(
                        main_window_clone.default_width(),
                        main_window_clone.default_height(),
                        main_window_clone.is_maximized()
                    )
                );
            });

        window_actions.add_action(open_help_overlay_action);
        application_actions.add_action(open_about_dialog_action);
        application_actions.add_action(close_application_action);

        let application_actions_group = application_actions.into_action_group();
        let window_actions_group = window_actions.into_action_group();
        main_window.insert_action_group("win", Some(&window_actions_group));
        main_window.insert_action_group("app", Some(&application_actions_group));

        let app = relm4::gtk_application();
        app.set_accelerators_for_action::<OpenHelpOverlay>(&["<primary>question"]);
        app.set_accelerators_for_action::<CloseApplication>(&["<primary>Q"]);

        let provider = gtk::CssProvider::new();
        provider.load_from_resource("/paveloom/apps/tidings/style.css");
        if let Some(display) = gdk::Display::default() {
            gtk::StyleContext::add_provider_for_display(
                &display,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
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
    let model = Model {
        settings: gio::Settings::new(APP_ID),
    };
    let app = RelmApp::new(model);

    info!("Tidings ({})", APP_ID);
    info!("Version: {} ({})", VERSION, PROFILE);
    info!("Datadir: {}", PKGDATADIR);

    app.run();
}
