//! Actions

use gtk::prelude::{GtkApplicationExt, WidgetExt};

use super::components::{
    about_dialog, add_directory_dialog, add_feed_dialog, content, help_overlay,
};
use super::Msg;
use relm4::actions::{AccelsPlus, RelmAction, RelmActionGroup};

relm4::new_action_group!(pub(super) WindowActionGroup, "win");
relm4::new_stateless_action!(pub(super) ShowHelpOverlay, WindowActionGroup, "show-help-overlay");
relm4::new_stateless_action!(pub(super) ShowAddFeedDialog, WindowActionGroup, "show-add-feed-dialog");
relm4::new_stateless_action!(pub(super) ShowAddDirectoryDialog, WindowActionGroup, "show-add-directory-dialog");
relm4::new_stateless_action!(pub(super) UpdateAllFeeds, WindowActionGroup, "update-all-feeds");

relm4::new_action_group!(pub(super) ApplicationActionGroup, "app");
relm4::new_stateless_action!(pub(super) ShowAboutDialog, ApplicationActionGroup, "about");
relm4::new_stateless_action!(pub(super) QuitApplication, ApplicationActionGroup, "quit");

/// Setup actions for the application window
pub(super) fn setup_actions(app_window: &adw::ApplicationWindow) {
    // Prepare action groups
    let window_actions = RelmActionGroup::<WindowActionGroup>::new();
    let application_actions = RelmActionGroup::<ApplicationActionGroup>::new();
    // Create the Show Add Feed Dialog action
    let show_add_feed_dialog_action: RelmAction<ShowAddFeedDialog> = RelmAction::new_stateless({
        move |_| {
            add_feed_dialog::BROKER.send(add_feed_dialog::Msg::Show);
        }
    });
    // Create the Show Add Directory Dialog action
    let show_add_directory_dialog_action: RelmAction<ShowAddDirectoryDialog> =
        RelmAction::new_stateless({
            move |_| {
                add_directory_dialog::BROKER.send(add_directory_dialog::Msg::Show);
            }
        });
    // Create the Update All Feeds action
    let update_all_feeds_action: RelmAction<UpdateAllFeeds> = RelmAction::new_stateless({
        move |_| {
            content::BROKER.send(content::Msg::ToggleUpdateAll);
        }
    });
    // Create the Show Help Overlay action
    let show_help_overlay_action: RelmAction<ShowHelpOverlay> = RelmAction::new_stateless({
        move |_| {
            help_overlay::BROKER.send(help_overlay::Msg::Show);
        }
    });
    // Create the Show About Dialog action
    let show_about_dialog_action: RelmAction<ShowAboutDialog> = RelmAction::new_stateless({
        move |_| {
            about_dialog::BROKER.send(about_dialog::Msg::Show);
        }
    });
    // Create the Quit Application action
    let quit_application_action: RelmAction<QuitApplication> = RelmAction::new_stateless({
        move |_| {
            super::BROKER.send(Msg::Quit);
        }
    });
    // Add the actions to the according group
    window_actions.add_action(show_help_overlay_action);
    window_actions.add_action(show_add_feed_dialog_action);
    window_actions.add_action(show_add_directory_dialog_action);
    window_actions.add_action(update_all_feeds_action);
    application_actions.add_action(show_about_dialog_action);
    application_actions.add_action(quit_application_action);
    // Insert the action groups into the window
    let window_actions_group = window_actions.into_action_group();
    let application_actions_group = application_actions.into_action_group();
    app_window.insert_action_group("win", Some(&window_actions_group));
    app_window.insert_action_group("app", Some(&application_actions_group));
}

/// Set accelerators for the actions
pub fn setup_accels(app: &impl GtkApplicationExt) {
    app.set_accelerators_for_action::<QuitApplication>(&["<primary>q"]);
    app.set_accelerators_for_action::<ShowHelpOverlay>(&["<primary>question"]);
    app.set_accelerators_for_action::<ShowAddFeedDialog>(&["<primary>a"]);
    app.set_accelerators_for_action::<ShowAddDirectoryDialog>(&["<primary>d"]);
    app.set_accelerators_for_action::<UpdateAllFeeds>(&["<primary>r"]);
}
