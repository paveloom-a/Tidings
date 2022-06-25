//! Tidings List View

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
            vec![ParamSpecString::new(
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
            )]
        });
        &PROPERTIES
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            "label" => {
                let label: String = value
                    .get()
                    .expect("The value needs to be of type `String`.");
                self.label.replace(label);
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "label" => self.label.borrow().to_value(),
            _ => unimplemented!(),
        }
    }
}

glib::wrapper! {
    pub struct Tiding(ObjectSubclass<GTiding>);
}

impl Tiding {
    /// Get a new tiding
    pub fn new(label: &str) -> Self {
        glib::Object::new(&[("label", &label.to_string())]).expect("Could not create `Tiding`.")
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
        for number in 0..=10 {
            let feed_object = Tiding::new(&number.to_string());
            store.append(&feed_object);
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
    let factory = gtk::SignalListItemFactory::new();
    factory.connect_setup(move |_, list_item| {
        // Create label
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));

        // Create expression describing `list_item->item->label`
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

        // Bind "number" to "label"
        label_expression.bind(&label, "label", Some(&label));
    });

    let filter = gtk::CustomFilter::new(move |obj| {
        // Downcast the object
        let tiding: &Tiding = obj
            .downcast_ref()
            .expect("The object needs to be of type `Tiding`.");

        // Get the label
        let _label: String = tiding.property("label");

        // Uncomment to only allow even numbers
        // _number % 2 == 0
        true
    });
    let filter_model = gtk::FilterListModel::new(Some(&model.store), Some(&filter));

    let sorter = gtk::CustomSorter::new(move |obj1, obj2| {
        // Downcast the objects
        let tiding_1: &Tiding = obj1
            .downcast_ref()
            .expect("The object needs to be of type `Tiding`.");
        let tiding_2: &Tiding = obj2
            .downcast_ref()
            .expect("The object needs to be of type `Tiding`.");

        // Get the labels
        let label_1: String = tiding_1.property("label");
        let label_2: String = tiding_2.property("label");

        // Reverse sorting order -> large strings come first
        label_2.cmp(&label_1).into()
    });
    let sort_model = gtk::SortListModel::new(Some(&filter_model), Some(&sorter));
    let selection_model = gtk::SingleSelection::new(Some(&sort_model));

    gtk::ListView::new(Some(&selection_model), Some(&factory))
}

#[allow(clippy::missing_docs_in_private_items)]
#[relm4_macros::widget(pub)]
impl relm4::Widgets<Model, AppModel> for Widgets {
    view! {
        list_view(model) -> gtk::ListView {
            set_single_click_activate: true,
            connect_activate(sender) => move |list_view, position| {
                // Get the model
                let model = list_view.model().expect("The model has to exist.");
                // Downcast the object
                let tiding: Tiding = model
                    .item(position)
                    .expect("The item has to exist.")
                    .downcast()
                    .expect("The item has to be an `Tiding`.");
                // Update the label
                tiding.update_string();
            }
        }
    }
}
