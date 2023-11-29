use std::any::Any;

use anathema_values::NodeId;

use crate::error::{Error, Result};

struct GiveThisASensibleName {
    inner: Vec<Box<dyn Fn() -> Box<dyn AnyView>>>,
}

impl GiveThisASensibleName {
    pub fn add<T, F>(&mut self, f: F)
    where
        F: 'static + Fn() -> T,
        T: 'static + View + std::fmt::Debug,
    {
        self.inner.push(Box::new(move || Box::new(f())));
    }

    pub fn blargh(&self, id: usize) -> Result<Box<dyn AnyView>> {
        match self.inner.get(id) {
            None => Err(Error::ViewNotFound),
            Some(f) => Ok(f()),
        }
    }
}

pub struct Views {
    inner: Vec<NodeId>,
}

pub trait View: Copy {
    type State: 'static;

    fn event(&self, event: (), state: &mut Self::State);

    fn make() -> Self;
}

pub trait AnyView: std::fmt::Debug {
    fn any_event(&mut self, ev: (), state: &mut dyn Any) -> ();
}

impl<T> AnyView for T
where
    T: View + std::fmt::Debug,
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
