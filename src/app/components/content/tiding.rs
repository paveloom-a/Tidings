//! Tiding

use adw::prelude::{ActionRowExt, PreferencesRowExt};
use gtk::traits::ListBoxRowExt;
use relm4::factory::{DynamicIndex, FactoryComponent, FactoryComponentSender};

/// Model
#[derive(Debug, Clone)]
pub struct Model {
    /// Title
    pub title: String,
}

/// Messages
#[derive(Debug)]
pub enum Msg {}

#[allow(clippy::missing_docs_in_private_items)]
#[relm4::factory(pub)]
impl FactoryComponent for Model {
    type CommandOutput = ();
    type Init = Model;
    type Input = Msg;
    type Output = ();
    type ParentMsg = super::Msg;
    type ParentWidget = gtk::ListBox;
    type Widgets = Widgets;
    view! {
        // Action Row
        adw::ActionRow {
            #[watch]
            set_title: &self.title,
            set_activatable: true,
            // Favicon
            add_prefix = &gtk::Image {
                set_icon_name: Some("emblem-shared-symbolic")
            },
        }
    }
    fn init_model(
        tiding: Self::Init,
        _index: &DynamicIndex,
        _sender: FactoryComponentSender<Self>,
    ) -> Self {
        // The callers should construct the variants themselves
        tiding
    }
    fn update(&mut self, msg: Self::Input, _sender: FactoryComponentSender<Self>) {
        match msg {}
    }
}
