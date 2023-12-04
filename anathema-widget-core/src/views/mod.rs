use std::any::Any;
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};

use anathema_values::hashmap::HashMap;
use anathema_values::NodeId;
use parking_lot::Mutex;

use crate::error::{Error, Result};

pub type ViewFn = dyn Fn() -> Box<dyn AnyView> + Send;

static TAB_INDEX: AtomicUsize = AtomicUsize::new(0);
static TAB_VIEWS: OnceLock<Mutex<BTreeSet<NodeId>>> = OnceLock::new();
static REGISTERED_VIEWS: OnceLock<Mutex<HashMap<String, Box<ViewFn>>>> = OnceLock::new();
static VIEWS: Mutex<BTreeSet<NodeId>> = Mutex::new(BTreeSet::new());

pub struct RegisteredViews;

impl RegisteredViews {
    pub fn add<T, F>(key: String, f: F)
    where
        F: Send + 'static + Fn() -> T,
        T: 'static + View,
    {
        REGISTERED_VIEWS
            .get_or_init(Default::default)
            .lock()
            .insert(key, Box::new(move || Box::new(f())));
    }

    pub fn get(id: &str) -> Result<Box<dyn AnyView>> {
        let views = REGISTERED_VIEWS.get_or_init(Default::default).lock();
        let view = views.get(id);

        match view {
            None => Err(Error::ViewNotFound),
            Some(f) => Ok(f()),
        }
    }
}

pub struct TabIndex;

impl TabIndex {
    fn next(&mut self) {
        TAB_INDEX.fetch_add(1, Ordering::Relaxed);

        let len = TAB_VIEWS.get_or_init(Default::default).lock().len();

        if TAB_INDEX.load(Ordering::Relaxed) == len {
            TAB_INDEX.store(0, Ordering::Relaxed);
        }
    }

    pub(crate) fn insert(node_id: NodeId) {
        TAB_VIEWS
            .get_or_init(Default::default)
            .lock()
            .insert(node_id);
    }

    fn remove(node_id: &NodeId) {
        TAB_VIEWS
            .get_or_init(Default::default)
            .lock()
            .remove(node_id);
    }

    pub(crate) fn remove_all<'a>(node_ids: impl Iterator<Item = &'a NodeId>) {
        let mut views = TAB_VIEWS.get_or_init(Default::default).lock();
        node_ids.for_each(|id| {
            views.remove(id);
        });
    }

    pub(crate) fn add_all<'a>(node_ids: impl Iterator<Item = &'a NodeId>) {
        let mut views = TAB_VIEWS.get_or_init(Default::default).lock();

        node_ids.cloned().for_each(|id| {
            views.insert(id);
        });
    }
}

pub struct Views;

impl Views {
    pub(crate) fn insert(node_id: NodeId) {
        VIEWS.lock().insert(node_id);
    }

    fn remove(node_id: &NodeId) {
        VIEWS.lock().remove(node_id);
    }
}

pub trait View {
    type State: 'static;

    fn event(&self, event: (), state: &mut Self::State);

    fn make() -> Self;
}

pub trait AnyView {
    fn any_event(&mut self, ev: (), state: &mut dyn Any) -> ();
}

impl<T> AnyView for T
where
    T: View,
{
    fn any_event(&mut self, ev: (), state: &mut dyn Any) -> () {
        if let Some(state) = state.downcast_mut::<T::State>() {
            self.event(ev, state);
        }
        ev
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::testing::view;

    #[test]
    fn events() {
        let v = view("a-view");
    }
}
