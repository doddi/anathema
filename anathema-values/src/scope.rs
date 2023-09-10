use std::borrow::Cow;
use std::ops::Deref;
use std::rc::Rc;
use std::str::FromStr;

use crate::hashmap::HashMap;
use crate::{NodeId, Path, State};

#[derive(Debug)]
pub enum Collection {
    Rc(Rc<[ScopeValue]>),
    State { path: Path, len: usize },
    Empty,
}

impl Collection {
    pub fn len(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::Rc(col) => col.len(),
            Self::State { len, .. } => *len,
        }
    }

    /// Increase the length of a state collection.
    /// This is a manual step for state bound lists
    /// as we don't access the entire list, only
    /// one value at a time when needed.
    pub fn add(&mut self) {
        if let Collection::State { len, .. } = self {
            *len += 1;
        }
    }

    /// Decrease the length of a state collection.
    /// This is a manual step (see `Self::add`)
    pub fn remove(&mut self) {
        if let Collection::State { len, .. } = self {
            *len -= 1;
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScopeValue {
    Static(Rc<str>),
    List(Rc<[ScopeValue]>),
    Dyn(Path),
}

impl<const N: usize> From<[ScopeValue; N]> for ScopeValue {
    fn from(arr: [ScopeValue; N]) -> Self {
        if N == 1 {
            arr.into_iter()
                .next()
                .expect("this is always going to be an array with a size of one")
        } else {
            ScopeValue::List(Rc::new(arr))
        }
    }
}

// TODO: add a testing flag for this
impl From<i32> for ScopeValue {
    fn from(s: i32) -> Self {
        Self::Static(s.to_string().into())
    }
}

// TODO: add a testing flag for this
impl From<String> for ScopeValue {
    fn from(s: String) -> Self {
        Self::Static(s.into())
    }
}

#[derive(Debug)]
pub struct Scope<'a> {
    inner: Vec<HashMap<Path, Cow<'a, ScopeValue>>>,
    // current: HashMap<Path, &'a ScopeValue>,
}

impl<'a> Scope<'a> {
    pub fn new(parent: Option<&'a Scope<'_>>) -> Self {
        Self {
            inner: vec![HashMap::new()],
            // current: HashMap::new(),
        }
    }

    pub fn scope(&mut self, path: Path, value: Cow<'a, ScopeValue>) {
        self.inner.last_mut().map(|values| values.insert(path, value));
    }

    pub fn push(&mut self) {
        self.inner.push(HashMap::new())
    }

    pub fn pop(&mut self) {
        self.inner.pop();
    }

    /// Scope a value for a collection.
    /// TODO: Review if the whole cloning business here makes sense
    pub fn scope_collection(&mut self, binding: Path, collection: &Collection, value_index: usize) {

        let value = match collection {
            Collection::Rc(list) => Cow::Owned(list[value_index].clone()),
            Collection::State { path, .. } => {
                let path = path.compose(value_index);
                Cow::Owned(ScopeValue::Dyn(path))
            }
            Collection::Empty => return,
        };

        self.scope(binding, value);
    }

    pub fn lookup(&self, path: &Path) -> Option<&ScopeValue> {
        self.inner
            .iter()
            .filter_map(|values| values.get(path).map(Deref::deref))
            .next()
    }

    pub fn lookup_list(&self, path: &Path) -> Option<Rc<[ScopeValue]>> {
        self.inner
            .iter()
            .filter_map(|values| values.get(path).map(Deref::deref))
            .filter_map(|value| match value {
                ScopeValue::List(list) => Some(list.clone()),
                _ => None,
            })
            .next()
    }

    // pub fn from_self(&'a self) -> Scope<'a> {
    //     panic!()
    //     // Scope::new(Some(self))
    // }
}

pub struct Context<'a, 'val> {
    pub state: &'a mut dyn State,
    pub scope: &'a mut Scope<'val>,
}

impl<'a, 'val> Context<'a, 'val> {
    pub fn new(state: &'a mut dyn State, scope: &'a mut Scope<'val>) -> Self {
        Self { state, scope }
    }

    /// Resolve a value based on paths.
    pub fn resolve(&self, value: &ScopeValue) -> ScopeValue {
        match value {
            ScopeValue::Static(_) => value.clone(),
            ScopeValue::Dyn(path) => match self.scope.lookup(path) {
                Some(lark @ ScopeValue::Dyn(p)) => self.resolve(lark),
                Some(_) => value.clone(),
                None => ScopeValue::Dyn(path.clone()),
            },
            ScopeValue::List(list) => {
                let values = list.iter().map(|v| self.resolve(v)).collect();
                ScopeValue::List(values)
            }
        }
    }

    /// Try to find the value in the current scope,
    /// if there is no value fallback to look for the value in the state.
    /// This will recursively lookup dynamic values
    pub fn get<T>(&self, path: &Path, node_id: Option<&NodeId>) -> Option<T>
    where
        T: for<'magic> TryFrom<&'magic str>,
    {
        match self.scope.lookup(&path) {
            Some(val) => match val {
                ScopeValue::Dyn(path) => self.get(path, node_id),
                ScopeValue::Static(s) => T::try_from(s).ok(),
                ScopeValue::List(_) => None,
            },
            None => self
                .state
                .get(&path, node_id.into())
                .and_then(|val| val.as_ref().try_into().ok()),
        }
    }

    pub fn attribute<T>(
        &self,
        key: impl AsRef<str>,
        node_id: Option<&NodeId>,
        attributes: &HashMap<String, ScopeValue>,
    ) -> Option<T>
    where
        T: for<'attr> TryFrom<&'attr str>,
    {
        let attrib = attributes.get(key.as_ref())?;

        match attrib {
            ScopeValue::Static(val) => val.as_ref().try_into().ok(),
            ScopeValue::Dyn(path) => self.get(path, node_id),
            _ => None,
        }
    }

    pub fn primitive<T>(
        &self,
        key: impl AsRef<str>,
        node_id: Option<&NodeId>,
        attributes: &HashMap<String, ScopeValue>,
    ) -> Option<T>
    where
        T: FromStr,
    {
        let attrib = attributes.get(key.as_ref())?;

        match attrib {
            ScopeValue::Static(val) => T::from_str(val.as_ref()).ok(),
            ScopeValue::Dyn(path) => self
                .get::<String>(path, node_id)
                .as_deref()
                .and_then(|s| T::from_str(s).ok()),
            _ => None,
        }
    }

    pub fn list_to_string(
        &self,
        list: &Rc<[ScopeValue]>,
        buffer: &mut String,
        node_id: Option<&NodeId>,
    ) {
        for val in list.iter() {
            match val {
                ScopeValue::List(list) => self.list_to_string(list, buffer, node_id),
                ScopeValue::Dyn(path) => buffer.push_str(&self.get_string(path, node_id)),
                ScopeValue::Static(s) => buffer.push_str(s),
            }
        }
    }

    pub fn get_string(&self, path: &Path, node_id: Option<&NodeId>) -> String {
        match self.scope.lookup(path) {
            Some(val) => match val {
                ScopeValue::Dyn(path) => self.get_string(path, node_id),
                ScopeValue::Static(s) => s.to_string(),
                ScopeValue::List(list) => {
                    let mut buffer = String::new();
                    self.list_to_string(list, &mut buffer, node_id);
                    buffer
                }
            },
            None => self
                .state
                .get(&path, node_id)
                .and_then(|val| val.as_ref().try_into().ok())
                .unwrap_or_else(String::new),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::testing::*;

    type Sub = usize;

    #[test]
    fn shadow_value() {
        let mut scope = Scope::new(None);
        scope.scope(
            "value".into(),
            Cow::Owned(ScopeValue::Static("hello world".into())),
        );

        // let mut inner = Scope::new(Some(&scope));
        scope.push();
        let value = scope.lookup(&"value".into()).unwrap();
        scope.scope("shadow".into(), Cow::Borrowed(value));

        // let ScopeValue::Static(lhs) = scope.lookup(&"shadow".into()).unwrap() else {
        //     panic!()
        // };
        // let ScopeValue::Static(rhs) = scope.lookup(&"value".into()).unwrap() else {
        //     panic!()
        // };
        // assert_eq!(lhs, rhs);
    }

    #[test]
    fn dynamic_attribute() {
        let mut state = TestState::new();
        let mut root = Scope::new(None);
        let mut ctx = Context::new(&mut state, &mut root);
        let mut attributes = HashMap::new();
        attributes.insert(
            "name".to_string(),
            ScopeValue::Dyn(Path::Key("name".into())),
        );

        let id = Some(123.into());
        let name: Option<String> = ctx.attribute("name", id.as_ref(), &attributes);

        assert_eq!("Dirk Gently", name.unwrap());
    }
}
