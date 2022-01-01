mod components;

use anyhow::Result;
use gettextrs::gettext;
use gtk::gio;
use gtk::prelude::{ApplicationWindowExt, GtkWindowExt, SettingsExt, WidgetExt};
use gtk::Justification;
use log::{debug, info, warn};
use relm4::{
    actions::{AccelsPlus, ActionGroupName, ActionName, RelmAction, RelmActionGroup},
    send, set_global_css, AppUpdate, Model, RelmApp, RelmComponent, Sender, WidgetPlus, Widgets,
};

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

#[relm4_macros::widget]
impl Widgets<AppModel, ()> for AppWidgets {
    view! {
        main_window = gtk::ApplicationWindow {
            set_title: Some(&gettext("Tidings")),
            set_icon_name: Some(APP_ID),
            set_default_width: model.settings.int("window-width"),
            set_default_height: model.settings.int("window-height"),
            set_maximized: model.settings.boolean("is-maximized"),
            set_help_overlay: Some(components.help_overlay.root_widget()),
            set_titlebar = Some(&gtk::HeaderBar) {
                pack_end = &gtk::MenuButton {
                    set_icon_name: "open-menu-symbolic",
                    set_menu_model: Some(&main_menu)
                }
            },
            connect_close_request(sender) => move |w| {
                send!(sender, AppMsg::Close(w.default_width(), w.default_height(), w.is_maximized()));
                gtk::Inhibit(true)
            },
            set_child = Some(&gtk::Label) {
                add_css_class: "title-header",
                set_margin_all: 5,
                set_hexpand: true,
                set_label: &gettext("Hello world!"),
                set_justify: Justification::Center,
            },
        }
    }

    menu! {
        main_menu: {
            "Preferences" => OpenPreferences,
            "Keyboard Shortcuts" => OpenHelpOverlay,
            "About Tidings" => OpenAboutDialog,
        }
    }

    fn post_init() {
        set_global_css(b".title-header { font-size: 36px; font-weight: bold; }");

        if PROFILE == "dev" {
            main_window.add_css_class("devel");
        }

        let app = relm4::gtk_application();
        app.set_accelerators_for_action::<OpenHelpOverlay>(&["<primary>question"]);
        app.set_accelerators_for_action::<CloseApplication>(&["<primary>Q"]);

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

        application_actions.add_action(open_about_dialog_action);
        application_actions.add_action(close_application_action);
        window_actions.add_action(open_help_overlay_action);

        let application_actions_group = application_actions.into_action_group();
        let window_actions_group = window_actions.into_action_group();
        main_window.insert_action_group("win", Some(&window_actions_group));
        main_window.insert_action_group("app", Some(&application_actions_group));
    }
}

relm4::new_action_group!(WindowActionGroup, "win");
relm4::new_action_group!(ApplicationActionGroup, "app");

relm4::new_statless_action!(OpenHelpOverlay, WindowActionGroup, "show-help-overlay");
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
