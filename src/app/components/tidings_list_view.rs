//! Tidings List View

use anyhow::{Context, Result};
use gtk::glib::{ParamFlags, ParamSpec, ParamSpecString, Value};
use gtk::prelude::{Cast, ListModelExt, ObjectExt, StaticType, ToValue};
use gtk::subclass::prelude::{ObjectImpl, ObjectSubclass};
use gtk::{gio, glib};
use once_cell::sync::Lazy;
use relm4::{ComponentUpdate, Sender};
use std::cell::RefCell;

use super::{AppModel, AppMsg};

/// Object holding the state
#[derive(Default)]
pub struct GTiding {
    /// Label
    label: RefCell<String>,
}

#[glib::object_subclass]
impl ObjectSubclass for GTiding {
    const NAME: &'static str = "Tiding";
    type Type = Tiding;
    type ParentType = glib::Object;
}

impl ObjectImpl for GTiding {
    fn properties() -> &'static [ParamSpec] {
        /// Properties
        static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
            vec![
                // Label of the tiding
                ParamSpecString::new(
                    // Name
                    "label",
                    // Nickname
                    "label",
                    // Short description
                    "label",
                    // Default value
                    Some(""),
                    // Flags
                    ParamFlags::READWRITE,
                ),
            ]
        });
        &PROPERTIES
    }
    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        if pspec.name() == "label" {
            let label: String = value.get().unwrap_or_else(|e| {
                log::error!("Couldn't unwrap the value of the `label` property");
                log::debug!("{e}");
                String::from("")
            });
            self.label.replace(label);
        } else {
            log::error!("Tried to set an unsupported property {value:?}");
        }
    }
    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        if pspec.name() == "label" {
            self.label.borrow().to_value()
        } else {
            log::error!("Tried to get an unsupported property");
            log::debug!("{pspec:?}");
            "".to_value()
        }
    }
}

glib::wrapper! {
    pub struct Tiding(ObjectSubclass<GTiding>);
}

impl Tiding {
    /// Initialize a tiding from the label
    pub fn new(label: &str) -> Result<Self> {
        glib::Object::new(&[("label", &label.to_owned())])
            .with_context(|| "Couldn't initialize a tiding")
    }
    /// Update the string
    pub fn update_string(self) {
        let label: String = self.property("label");
        self.set_property("label", format!("{}!", label));
    }
}

/// Model
pub struct Model {
    /// List Store
    store: gio::ListStore,
}

impl relm4::Model for Model {
    type Msg = ();
    type Widgets = Widgets;
    type Components = ();
}

impl ComponentUpdate<AppModel> for Model {
    fn init_model(_parent_model: &AppModel) -> Self {
        let store = gio::ListStore::new(Tiding::static_type());
        // Add fake tidings with numbers as labels
        for number in 0_usize..=10_usize {
            let label = &number.to_string();
            match Tiding::new(label) {
                Ok(t) => {
                    store.append(&t);
                }
                Err(e) => {
                    log::error!("Couldn't create a tiding from the label {label}");
                    log::debug!("{e}");
                }
            }
        }
        Self { store }
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
            Tiding::static_type(),
            Some(&feed_object_expression),
            "label",
        );
        // Bind the labels
        label_expression.bind(&label, "label", Some(&label));
    });
    // Prepare a filter
    let filter = gtk::CustomFilter::new(move |obj| {
        // Downcast the object
        if let Some(tiding) = obj.downcast_ref::<Tiding>() {
            // Get the label
            let _label: String = tiding.property("label");
            true
        } else {
            log::error!("Couldn't unwrap the object in the filter");
            false
        }
    });
    // Create a filter model
    let filter_model = gtk::FilterListModel::new(Some(&model.store), Some(&filter));
    // Prepare a sorter
    let sorter = gtk::CustomSorter::new(move |obj_1, obj_2| {
        // Downcast the objects
        if let Some(tiding_1) = obj_1.downcast_ref::<Tiding>() {
            if let Some(tiding_2) = obj_2.downcast_ref::<Tiding>() {
                // Get the labels
                let label_1: String = tiding_1.property("label");
                let label_2: String = tiding_2.property("label");
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
                        if let Ok(tiding) = item.downcast::<Tiding>() {
                            // Update the label
                            tiding.update_string();
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
