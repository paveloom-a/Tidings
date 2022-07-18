//! Tidings

mod list;

use gtk::prelude::{BoxExt, Cast, ListModelExt, ObjectExt, OrientableExt, StaticType, WidgetExt};
use relm4::{ComponentUpdate, Sender};

use crate::app::actions::{ShowAboutDialog, ShowHelpOverlay};
use list::{Item, List};

/// Model
pub struct Model {
    /// List of items
    list: List,
}

/// Messages
pub enum Msg {}

impl relm4::Model for Model {
    type Msg = Msg;
    type Widgets = Widgets;
    type Components = ();
}

impl ComponentUpdate<super::Model> for Model {
    fn init_model(_parent_model: &super::Model) -> Self {
        // Initialize a list
        let list = List::new(Item::static_type());
        // Add fake tidings with numbers as labels
        for number in 0_usize..=10_usize {
            let label = &number.to_string();
            if let Some(item) = Item::new(label) {
                list.append(&item);
            }
        }
        Self { list }
    }
    fn update(
        &mut self,
        _msg: Msg,
        _components: &(),
        _sender: Sender<Msg>,
        _parent_sender: Sender<super::Msg>,
    ) {
    }
}

/// Get a `ListView` from the model
fn list_view(model: &Model) -> gtk::ListView {
    // Prepare a factory
    let factory = gtk::SignalListItemFactory::new();
    factory.connect_setup(move |_, list_item| {
        // Attach a label to the list item
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));
        // Create expressions describing `list_item -> item -> label`
        let list_item_expression = gtk::ConstantExpression::new(list_item);
        let feed_object_expression = gtk::PropertyExpression::new(
            gtk::ListItem::static_type(),
            Some(&list_item_expression),
            "item",
        );
        let label_expression = gtk::PropertyExpression::new(
            Item::static_type(),
            Some(&feed_object_expression),
            "label",
        );
        // Bind the labels
        label_expression.bind(&label, "label", Some(&label));
    });
    // Prepare a filter
    let filter = gtk::CustomFilter::new(move |obj| {
        // Downcast the object
        if let Some(item) = obj.downcast_ref::<Item>() {
            // Get the label
            let _label: String = item.property("label");
            return true;
        }
        false
    });
    // Create a filter model
    let filter_model = gtk::FilterListModel::new(Some(&model.list), Some(&filter));
    // Prepare a sorter
    let sorter = gtk::CustomSorter::new(move |obj_1, obj_2| {
        // Downcast the objects
        if let Some(item_1) = obj_1.downcast_ref::<Item>() {
            if let Some(item_2) = obj_2.downcast_ref::<Item>() {
                // Get the labels
                let label_1: String = item_1.property("label");
                let label_2: String = item_2.property("label");
                // Reverse the sorting order (large strings come first)
                return label_2.cmp(&label_1).into();
            }
        }
        // Default to
        gtk::Ordering::Larger
    });
    // Create a sorter model
    let sort_model = gtk::SortListModel::new(Some(&filter_model), Some(&sorter));
    // Create a selection model
    let selection_model = gtk::SingleSelection::new(Some(&sort_model));
    // Create a List View
    gtk::ListView::new(Some(&selection_model), Some(&factory))
}

/// Connect the activate event of the List View
fn list_view_connect_activate(_sender: &Sender<Msg>, list_view: &gtk::ListView, position: u32) {
    // Get the model
    if let Some(model) = list_view.model() {
        // Get the item at the position
        if let Some(item) = model.item(position) {
            // Downcast the object
            if let Ok(item) = item.downcast::<Item>() {
                // Update the label
                item.update_string();
            }
        }
    }
}

#[allow(clippy::missing_docs_in_private_items)]
#[relm4_macros::widget(pub)]
impl relm4::Widgets<Model, super::Model> for Widgets {
    view! {
        // Box
        gtk::Box {
            set_width_request: 365,
            set_orientation: gtk::Orientation::Vertical,
            // Header Bar
            append: header_bar = &adw::HeaderBar {
                // Title
                set_title_widget = Some(&adw::WindowTitle) {
                    set_title: "Tidings"
                },
                // Menu Button
                pack_end = &gtk::MenuButton {
                    set_icon_name: "open-menu-symbolic",
                    set_menu_model: Some(&main_menu),
                },
            },
            // Scrolled Window
            append = &gtk::ScrolledWindow {
                set_hscrollbar_policy: gtk::PolicyType::Never,
                set_hexpand: true,
                set_vexpand: true,
                // List View
                set_child = Some(&list_view(model) -> gtk::ListView) {
                    set_single_click_activate: true,
                    connect_activate(sender) => move |list_view, position| {
                        list_view_connect_activate(&sender, list_view, position);
                    }
                }
            }
        }
    }
    menu! {
        main_menu: {
            "Keyboard Shortcuts" => ShowHelpOverlay,
            "About Tidings" => ShowAboutDialog,
        }
    }
}
