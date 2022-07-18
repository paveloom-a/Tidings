//! Actions

use gtk::prelude::WidgetExt;

use super::components::{about_dialog, add_directory_dialog, add_feed_dialog, help_overlay};
use super::{Components, Msg};
use relm4::actions::{AccelsPlus, RelmAction, RelmActionGroup};
use relm4::Sender;

relm4::new_action_group!(pub(super) WindowActionGroup, "win");
relm4::new_stateless_action!(pub(super) ShowHelpOverlay, WindowActionGroup, "show-help-overlay");
relm4::new_stateless_action!(pub(super) ShowAddFeedDialog, WindowActionGroup, "show-add-feed-dialog");
relm4::new_stateless_action!(pub(super) ShowAddDirectoryDialog, WindowActionGroup, "show-add-directory-dialog");

relm4::new_action_group!(pub(super) ApplicationActionGroup, "app");
relm4::new_stateless_action!(pub(super) ShowAboutDialog, ApplicationActionGroup, "about");
relm4::new_stateless_action!(pub(super) CloseApplication, ApplicationActionGroup, "quit");

/// Setup actions for the application window
pub(super) fn setup_actions(
    app_window: &adw::ApplicationWindow,
    components: &Components,
    sender: &Sender<Msg>,
) {
    // Prepare action groups
    let window_actions = RelmActionGroup::<WindowActionGroup>::new();
    let application_actions = RelmActionGroup::<ApplicationActionGroup>::new();
    // Create the Show Help Overlay action
    let show_help_overlay_action: RelmAction<ShowHelpOverlay> = RelmAction::new_stateless({
        let sender = components.help_overlay.sender();
        move |_| {
            sender.send(help_overlay::Msg::Show).ok();
        }
    });
    // Create the Show About Dialog action
    let show_about_dialog_action: RelmAction<ShowAboutDialog> = RelmAction::new_stateless({
        let sender = components.about_dialog.sender();
        move |_| {
            sender.send(about_dialog::Msg::Show).ok();
        }
    });
    // Create the Close Application action
    let close_application_action: RelmAction<CloseApplication> = RelmAction::new_stateless({
        let sender = sender.clone();
        move |_| {
            sender.send(Msg::Close).ok();
        }
    });
    // Create the Show Add Feed Dialog action
    let show_add_feed_dialog_action: RelmAction<ShowAddFeedDialog> = RelmAction::new_stateless({
        let sender = components.add_feed_dialog.sender();
        move |_| {
            sender.send(add_feed_dialog::Msg::Show).ok();
        }
    });
    // Create the Show Add Directory Dialog action
    let show_add_directory_dialog_action: RelmAction<ShowAddDirectoryDialog> =
        RelmAction::new_stateless({
            let sender = components.add_directory_dialog.sender();
            move |_| {
                sender.send(add_directory_dialog::Msg::Show).ok();
            }
        });
    // Add the actions to the according group
    window_actions.add_action(show_help_overlay_action);
    window_actions.add_action(show_add_feed_dialog_action);
    window_actions.add_action(show_add_directory_dialog_action);
    application_actions.add_action(show_about_dialog_action);
    application_actions.add_action(close_application_action);
    // Insert the action groups into the window
    let window_actions_group = window_actions.into_action_group();
    let application_actions_group = application_actions.into_action_group();
    app_window.insert_action_group("win", Some(&window_actions_group));
    app_window.insert_action_group("app", Some(&application_actions_group));
    // Set accelerators for the actions
    let app = relm4::gtk_application();
    app.set_accelerators_for_action::<CloseApplication>(&["<primary>Q"]);
    app.set_accelerators_for_action::<ShowHelpOverlay>(&["<primary>question"]);
    app.set_accelerators_for_action::<ShowAddFeedDialog>(&["<primary>a"]);
    app.set_accelerators_for_action::<ShowAddDirectoryDialog>(&["<primary>d"]);
}
