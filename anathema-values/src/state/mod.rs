use std::ops::Deref;

pub use self::value::{Change, StateValue};
use crate::{NodeId, Path, ValueRef};

mod value;

pub trait State {
    /// Get a value reference from the state
    fn get(&self, key: &Path, node_id: Option<&NodeId>) -> ValueRef<'_>;

    #[doc(hidden)]
    fn get_value(&self, _: Option<&NodeId>) -> ValueRef<'_> {
        ValueRef::Empty
    }
}

impl State for Box<dyn State> {
    fn get(&self, key: &Path, node_id: Option<&NodeId>) -> ValueRef<'_> {
        self.deref().get(key, node_id)
    }
}

/// This exists so you can have a view with a default state of a unit
impl State for () {
    fn get(&self, key: &Path, node_id: Option<&NodeId>) -> ValueRef<'_> {
        ValueRef::Empty
    }
}
