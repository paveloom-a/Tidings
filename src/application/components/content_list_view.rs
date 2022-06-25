use gtk::glib::{Object, ParamFlags, ParamSpec, ParamSpecString, Value};
use gtk::prelude::{Cast, ListModelExt, ObjectExt, StaticType, ToValue};
use gtk::subclass::prelude::{ObjectImpl, ObjectSubclass};
use gtk::{gio, glib};
use once_cell::sync::Lazy;
use relm4::{ComponentUpdate, Model, Sender, Widgets};
use std::cell::RefCell;

use super::{AppModel, AppMsg};

// Object holding the state
#[derive(Default)]
pub struct GContentObject {
    label: RefCell<String>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for GContentObject {
    const NAME: &'static str = "TidingsContentObject";
    type Type = ContentObject;
    type ParentType = glib::Object;
}

// Trait shared by all GObjects
impl ObjectImpl for GContentObject {
    fn properties() -> &'static [ParamSpec] {
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
        PROPERTIES.as_ref()
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
    pub struct ContentObject(ObjectSubclass<GContentObject>);
}

impl ContentObject {
    pub fn new(label: &str) -> Self {
        Object::new(&[("label", &label.to_string())]).expect("Could not create `ContentObject`.")
    }

    pub fn update_string(self) {
        let label: String = self.property("label");
        self.set_property("label", format!("{}!", label));
    }
}

pub struct ContentModel {
    store: gio::ListStore,
}

pub enum ContentMsg {}

impl Model for ContentModel {
    type Msg = ContentMsg;
    type Widgets = FeedsWidgets;
    type Components = ();
}

impl ComponentUpdate<AppModel> for ContentModel {
    fn init_model(_parent_model: &AppModel) -> Self {
        let store = gio::ListStore::new(ContentObject::static_type());
        for number in 0..=10 {
            let feed_object = ContentObject::new(&number.to_string());
            store.append(&feed_object);
        }

        ContentModel { store }
    }

    fn update(
        &mut self,
        _msg: ContentMsg,
        _components: &(),
        _sender: Sender<ContentMsg>,
        _parent_sender: Sender<AppMsg>,
    ) {
    }
}

fn list_view(model: &ContentModel) -> gtk::ListView {
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
            ContentObject::static_type(),
            Some(&feed_object_expression),
            "label",
        );

        // Bind "number" to "label"
        label_expression.bind(&label, "label", Some(&label));
    });

    let filter = gtk::CustomFilter::new(move |obj| {
        // Get `ContentObject` from `glib::Object`
        let feed_object: &ContentObject = obj
            .downcast_ref()
            .expect("The object needs to be of type `ContentObject`.");

        // Get property "label" from `ContentObject`
        let _label: String = feed_object.property("label");

        // Uncomment to only allow even numbers
        // _number % 2 == 0
        true
    });
    let filter_model = gtk::FilterListModel::new(Some(&model.store), Some(&filter));

    let sorter = gtk::CustomSorter::new(move |obj1, obj2| {
        // Get `ContentObject` from `glib::Object`
        let feed_object_1: &ContentObject = obj1
            .downcast_ref()
            .expect("The object needs to be of type `ContentObject`.");
        let feed_object_2: &ContentObject = obj2
            .downcast_ref()
            .expect("The object needs to be of type `ContentObject`.");

        // Get property "label" from `ContentObject`
        let label_1: String = feed_object_1.property("label");
        let label_2: String = feed_object_2.property("label");

        // Reverse sorting order -> large strings come first
        label_2.cmp(&label_1).into()
    });
    let sort_model = gtk::SortListModel::new(Some(&filter_model), Some(&sorter));
    let selection_model = gtk::SingleSelection::new(Some(&sort_model));

    gtk::ListView::new(Some(&selection_model), Some(&factory))
}

#[relm4_macros::widget(pub)]
impl Widgets<ContentModel, AppModel> for FeedsWidgets {
    view! {
        list_view(model) -> gtk::ListView {
            set_single_click_activate: true,
            connect_activate(sender) => move |list_view, position| {
                // Get `ContentObject` from model
                let model = list_view.model().expect("The model has to exist.");
                let content_object: ContentObject = model
                    .item(position)
                    .expect("The item has to exist.")
                    .downcast()
                    .expect("The item has to be an `ContentObject`.");

                // Update "label" of `ContentObject`
                content_object.update_string();
            }
        }
    }
}
