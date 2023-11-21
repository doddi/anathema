use std::ops::Deref;

pub use self::value::{Change, StateValue};
use crate::{NodeId, Path, ValueRef};

mod value;

pub trait State {
    /// Get a value reference from the state
    fn get(&self, key: &Path, node_id: Option<&NodeId>) -> ValueRef<'_>;

    #[doc(hidden)]
    fn __anathema_get(&self, key: &Path, node_id: Option<&NodeId>) -> ValueRef<'_>  {
        self.get(key, node_id)
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

/// This is a generic trait for everything that 
/// doesn't implement `State` (otherwise non StateValue<T>s would fail to compile)
pub trait BlanketGet {
    fn __anathema_get_value(&self, node_id: Option<&NodeId>) -> ValueRef<'static> {
        panic!("get value: temporary solution: panic for now to ensure values aren't missed");
        ValueRef::Empty
    }

    fn __anathema_get<'a>(&self, key: &'a Path, node_id: Option<&NodeId>) -> ValueRef<'static> {
        panic!("get: temporary solution: panic for now to ensure values aren't missed");
        ValueRef::Empty
    }

    fn __anathema_subscribe(&self, node_id: NodeId) {}
}

impl<T> BlanketGet for &T {}

