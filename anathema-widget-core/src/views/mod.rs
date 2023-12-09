use std::any::Any;
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};
use std::fmt::Debug;

use anathema_values::hashmap::HashMap;
use anathema_values::{NodeId, State};
use parking_lot::Mutex;
use kempt::Set;

use crate::{Event, Nodes};
use crate::error::{Error, Result};

pub type ViewFn = dyn Fn() -> Box<dyn AnyView> + Send;

static TAB_INDEX: AtomicUsize = AtomicUsize::new(0);
static TAB_VIEWS: Mutex<Set<NodeId>> = Mutex::new(Set::new());
static VIEWS: Mutex<Set<NodeId>> = Mutex::new(Set::new());
static REGISTERED_VIEWS: OnceLock<Mutex<HashMap<String, Box<ViewFn>>>> = OnceLock::new();

pub struct RegisteredViews;

impl RegisteredViews {
    pub fn add<T, F>(key: String, f: F)
    where
        F: Send + 'static + Fn() -> T,
        T: 'static + View + Debug,
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
    pub fn next() {
        TAB_INDEX.fetch_add(1, Ordering::Relaxed);

        let len = TAB_VIEWS.lock().len();

        if TAB_INDEX.load(Ordering::Relaxed) == len {
            TAB_INDEX.store(0, Ordering::Relaxed);
        }
    }

    pub(crate) fn insert(node_id: NodeId) {
        TAB_VIEWS
            .lock()
            .insert(node_id);
    }

    fn remove(node_id: &NodeId) {
        TAB_VIEWS
            .lock()
            .remove(node_id);
    }

    pub(crate) fn remove_all<'a>(node_ids: impl Iterator<Item = &'a NodeId>) {
        let mut views = TAB_VIEWS.lock();
        node_ids.for_each(|id| {
            views.remove(id);
        });
    }

    pub(crate) fn add_all<'a>(node_ids: impl Iterator<Item = &'a NodeId>) {
        let mut views = TAB_VIEWS.lock();

        node_ids.cloned().for_each(|id| {
            views.insert(id);
        });
    }

    pub fn current() -> Option<NodeId> {
        let index = TAB_INDEX.load(Ordering::Relaxed);
        let all = TAB_VIEWS.lock().clone();
        TAB_VIEWS.lock().member(index).cloned()
    }
}

pub struct Views;

impl Views {
    pub fn all() -> Vec<NodeId> {
        VIEWS.lock().iter().cloned().collect()
    }

    pub(crate) fn insert(node_id: NodeId) {
        VIEWS.lock().insert(node_id);
    }

    fn remove(node_id: &NodeId) {
        VIEWS.lock().remove(node_id);
    }
}

pub trait View {
    type State: 'static;

    fn on_event(&mut self, event: Event, nodes: &mut Nodes<'_>) {
    }

    fn make() -> Self;

    fn get_state(&self) -> &dyn State {
        &()
    }
}

pub trait AnyView : Debug {
    fn on_any_event(&mut self, ev: Event, nodes: &mut Nodes<'_>);

    fn get_any_state(&self) -> &dyn State;
}

impl<T> AnyView for T
where
    T: View + Debug,
{
    fn on_any_event(&mut self, event: Event, nodes: &mut Nodes<'_>) {
        self.on_event(event, nodes);
    }

    fn get_any_state(&self) -> &dyn State {
        self.get_state()
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
