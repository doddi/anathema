use std::any::Any;

pub struct Views {
}

pub trait View {
    type State: 'static;

    fn event(&self, event: (), state: &mut Self::State);
}

pub trait AnyView : std::fmt::Debug {
    fn any_event(&mut self, ev: (), state: &mut dyn Any) -> ();
}

impl<T> AnyView for T where T:View + std::fmt::Debug {
    fn any_event(&mut self, ev: (), state: &mut dyn Any) -> () {
        if let Some(state) = state.downcast_mut::<T::State>() {
            self.event(ev, state);
        }
        ev
    }
}

#[cfg(test)]
mod test {
    use crate::testing::view;

    use super::*;

    #[test]
    fn events() {
        let v = view("a-view");
    }
}
