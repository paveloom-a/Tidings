mod components;

use adw::prelude::GtkWindowExt;
use anyhow::Result;
use gtk::prelude::{SettingsExt, WidgetExt};
use gtk::{gdk, gio};
use log::{debug, error, info, warn};
use relm4::actions::{AccelsPlus, ActionGroupName, ActionName, RelmAction, RelmActionGroup};
use relm4::{send, AppUpdate, Model, RelmApp, RelmComponent, Sender, Widgets};

use super::config::{APP_ID, PKGDATADIR, PROFILE, VERSION};
use components::{AboutDialogModel, AboutDialogMsg, HelpOverlayModel, HelpOverlayMsg};

struct AppModel {
    settings: gio::Settings,
}

impl AppModel {
    fn save_settings(&self, width: i32, height: i32, is_maximized: bool) -> Result<()> {
        self.settings.set_int("window-width", width)?;
        self.settings.set_int("window-height", height)?;
        self.settings.set_boolean("is-maximized", is_maximized)?;
        Ok(())
    }
}

enum AppMsg {
    OpenAboutDialog,
    OpenHelpOverlay,
    Close(i32, i32, bool),
}

#[derive(relm4_macros::Components)]
struct AppComponents {
    about_dialog: RelmComponent<AboutDialogModel, AppModel>,
    help_overlay: RelmComponent<HelpOverlayModel, AppModel>,
}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = AppComponents;
}

impl AppUpdate for AppModel {
    fn update(&mut self, msg: AppMsg, components: &AppComponents, _sender: Sender<AppMsg>) -> bool {
        match msg {
            AppMsg::OpenAboutDialog => {
                components.about_dialog.send(AboutDialogMsg::Open).unwrap();
                true
            }
            AppMsg::OpenHelpOverlay => {
                components.help_overlay.send(HelpOverlayMsg::Open).unwrap();
                true
            }
            AppMsg::Close(width, height, is_maximized) => {
                if let Err(err) = self.save_settings(width, height, is_maximized) {
                    warn!("Failed to save window state");
                    debug!("{}", &err);
                }
                false
            }
        }
    }
}

fn main_window() -> adw::ApplicationWindow {
    if let Some(mw) = gtk::Builder::from_resource("/paveloom/apps/tidings/ui/main-window.ui")
        .object::<adw::ApplicationWindow>("main_window")
    {
        mw
    } else {
        error!("Failed to load Main Window UI");
        adw::ApplicationWindowBuilder::default().build()
    }
}

#[relm4_macros::widget]
impl Widgets<AppModel, ()> for AppWidgets {
    view! {
        main_window = main_window() -> adw::ApplicationWindow {
            set_default_width: model.settings.int("window-width"),
            set_default_height: model.settings.int("window-height"),
            set_maximized: model.settings.boolean("is-maximized"),
            connect_close_request(sender) => move |w| {
                send!(sender, AppMsg::Close(w.default_width(), w.default_height(), w.is_maximized()));
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
            RelmAction::new_statelesss(move |_| {
                send!(sender_clone, AppMsg::OpenHelpOverlay);
            });

        let sender_clone = sender.clone();
        let open_about_dialog_action: RelmAction<OpenAboutDialog> =
            RelmAction::new_statelesss(move |_| {
                send!(sender_clone, AppMsg::OpenAboutDialog);
            });

        let main_window_clone = main_window.clone();
        let close_application_action: RelmAction<CloseApplication> =
            RelmAction::new_statelesss(move |_| {
                send!(
                    sender,
                    AppMsg::Close(
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
relm4::new_statless_action!(OpenHelpOverlay, WindowActionGroup, "show-help-overlay");

relm4::new_action_group!(ApplicationActionGroup, "app");
relm4::new_statless_action!(OpenAboutDialog, ApplicationActionGroup, "about");
relm4::new_statless_action!(OpenPreferences, ApplicationActionGroup, "preferences");
relm4::new_statless_action!(CloseApplication, ApplicationActionGroup, "quit");

pub fn run() {
    let model = AppModel {
        settings: gio::Settings::new(APP_ID),
    };
    let app = RelmApp::new(model);

    info!("Tidings ({})", APP_ID);
    info!("Version: {} ({})", VERSION, PROFILE);
    info!("Datadir: {}", PKGDATADIR);

    app.run();
}
