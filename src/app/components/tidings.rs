//! Tidings

mod list;

use gtk::prelude::{Cast, ListModelExt, ObjectExt, StaticType};
use relm4::{ComponentUpdate, Sender};

use super::{AppModel, AppMsg};
use list::{Item, List};

/// Model
pub struct Model {
    /// List of items
    list: List,
}

impl relm4::Model for Model {
    type Msg = ();
    type Widgets = Widgets;
    type Components = ();
}

impl ComponentUpdate<AppModel> for Model {
    fn init_model(_parent_model: &AppModel) -> Self {
        // Initialize a list
        let list = List::new(Item::static_type());
        // Add fake tidings with numbers as labels
        for number in 0_usize..=10_usize {
            let label = &number.to_string();
            match Item::new(label) {
                Ok(t) => {
                    list.append(&t);
                }
                Err(e) => {
                    log::error!("Couldn't create a tiding from the label {label}");
                    log::debug!("{e}");
                }
            }
        }
        Self { list }
    }
    fn update(
        &mut self,
        _msg: (),
        _components: &(),
        _sender: Sender<()>,
        _parent_sender: Sender<AppMsg>,
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
            true
        } else {
            log::error!("Couldn't unwrap the object in the filter");
            false
        }
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
                label_2.cmp(&label_1).into()
            } else {
                log::error!("Couldn't unwrap the second object in the sorter");
                gtk::Ordering::Larger
            }
        } else {
            log::error!("Couldn't unwrap the first object in the sorter");
            gtk::Ordering::Larger
        }
    });
    // Create a sorter model
    let sort_model = gtk::SortListModel::new(Some(&filter_model), Some(&sorter));
    // Create a selection model
    let selection_model = gtk::SingleSelection::new(Some(&sort_model));
    // Create a List View
    gtk::ListView::new(Some(&selection_model), Some(&factory))
}

#[allow(clippy::missing_docs_in_private_items)]
#[relm4_macros::widget(pub)]
impl relm4::Widgets<Model, AppModel> for Widgets {
    view! {
        list_view(model) -> gtk::ListView {
            set_single_click_activate: true,
            connect_activate(sender) => move |list_view, position| {
                let _sender = &sender;
                // Get the model
                if let Some(model) = list_view.model() {
                    // Get the item at the position
                    if let Some(item) = model.item(position) {
                        // Downcast the object
                        if let Ok(item) = item.downcast::<Item>() {
                            // Update the label
                            item.update_string();
                        } else {
                            log::error!("Couldn't downcast the object");
                        }
                    } else {
                        log::error!("Couldn't get the item at the position {position}");
                    }
                } else {
                    log::error!("Couldn't unwrap the model");
                }
            }
        }
    }
}
