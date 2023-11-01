use std::cell::RefCell;

pub use self::id::NodeId;
pub use self::list::List;
pub use self::path::Path;
pub use self::scope::{Context, LocalScope};
pub use self::slab::Slab;
pub use self::state::{Change, State, StateValue};
pub use self::value_expr::{ValueResolver, Resolver, Deferred, ValueExpr};
pub use self::value::{ValueRef, Num, Owned};
pub use self::collection::Collection;

pub mod hashmap;
mod path;

mod collection;
mod id;
mod list;
mod map;
mod scope;
mod slab;
mod state;
mod value;
mod value_expr;

pub type Attributes = hashmap::HashMap<String, ValueExpr>;

thread_local! {
    static DIRTY_NODES: RefCell<Vec<(NodeId, Change)>> = Default::default();
    static REMOVED_NODES: RefCell<Vec<NodeId>> = Default::default();
}

pub fn drain_dirty_nodes() -> Vec<(NodeId, Change)> {
    DIRTY_NODES.with(|nodes| nodes.borrow_mut().drain(..).collect())
}

pub fn remove_node(node: NodeId) {
    REMOVED_NODES.with(|nodes| nodes.borrow_mut().push(node));
}

#[cfg(any(feature = "testing", test))]
pub mod testing;

#[derive(Debug)]
pub enum Value<T> {
    Dyn { inner: Option<T>, expr: ValueExpr },
    Static(T),
    Empty,
}

impl<T> Value<T>
where
    T: for<'b> TryFrom<ValueRef<'b>>,
{
    pub fn new(expr: ValueExpr, context: &Context<'_, '_>, node_id: Option<&NodeId>) -> Self {
        let mut resolver = Resolver::new(context, node_id);

        let inner = expr.eval(&mut resolver);

        // Here the inner value might not be a simple deferred value.
        // We need to establish that the value is deferred before we get here.
        // Maybe we can check this when we insert the value into wherever we insert it?

        match resolver.is_deferred() {
            true => Self::Dyn { inner: None, expr },
            false => match inner {
                Some(ValueRef::Deferred(_)) => Self::Dyn { inner: None, expr },
                Some(val) => match T::try_from(val) {
                    Ok(val) => Self::Static(val),
                    Err(_) => Self::Empty,
                }
                None => Self::Empty,
            },
        }
    }

    pub fn value(&self) -> Option<&T> {
        match self {
            Self::Static(val) => Some(val),
            Self::Dyn { inner, .. } => inner.as_ref(),
            _ => None,
        }
    }
}

impl Value<bool> {
    pub fn is_true(&self) -> bool {
        match self {
            Self::Dyn { inner, .. } => inner.unwrap_or(false),
            Self::Static(b) => *b,
            Self::Empty => false,
        }
    }
}

impl Value<String> {
    pub fn string(&self) -> &str {
        static EMPTY: &str = "";
        match self {
            Self::Static(s) => s,
            Self::Dyn { inner: Some(s), .. } => s,
            Self::Dyn { inner: None, .. } => EMPTY,
            Self::Empty => EMPTY,
        }
    }
}

impl DynValue for Value<String> {
    type Value = String;

    fn init(context: &Context<'_, '_>, node_id: Option<&NodeId>, expr: &ValueExpr) -> Option<Self> {
        let mut resolver = Resolver::new(context, node_id);
        let inner = resolver.resolve_string(expr);

        match resolver.is_deferred() {
            true => Some(Self::Dyn { inner, expr: expr.clone() }),
            false => match inner {
                Some(val) => Some(Self::Static(val)),
                None => None,
            }
        }
    }

    fn resolve(&mut self, context: &Context<'_, '_>, node_id: Option<&NodeId>) {
        match self {
            Self::Dyn { inner, expr } => {
                *inner = Resolver::new(context, node_id).resolve_string(expr)
            }
            _ => {}
        }
    }
}

pub trait DynValue {
    type Value: for<'b> TryFrom<ValueRef<'b>>;

    fn init(context: &Context<'_, '_>, node_id: Option<&NodeId>, expr: &ValueExpr) -> Option<Self> where Self: Sized;

    fn resolve(&mut self, context: &Context<'_, '_>, node_id: Option<&NodeId>);
}

macro_rules! value_resolver_for_basetype {
    ($t:ty) => {
        impl DynValue for Value<$t> {
            type Value = $t;

            fn init(context: &Context<'_, '_>, node_id: Option<&NodeId>, expr: &ValueExpr) -> Option<Self> {
                let mut resolver = Resolver::new(context, node_id);
                let inner = expr.eval(&mut resolver).and_then(|v| Self::Value::try_from(v).ok());

                match resolver.is_deferred() {
                    true => Some(Self::Dyn { inner, expr: expr.clone() }),
                    false => match inner {
                        Some(val) => Some(Self::Static(val)),
                        None => None,
                    }
                }
            }

            fn resolve(&mut self, context: &Context<'_, '_>, node_id: Option<&NodeId>) {
                match self {
                    Self::Dyn { inner, expr } => {
                        *inner = expr
                            .eval(&mut Resolver::new(context, node_id))
                            .and_then(|v| Self::Value::try_from(v).ok())
                    }
                    _ => {}
                }
            }
        }
    };
}

value_resolver_for_basetype!(bool);
// value_resolver_for_basetype!(Color);

value_resolver_for_basetype!(usize);
value_resolver_for_basetype!(u64);
value_resolver_for_basetype!(u32);
value_resolver_for_basetype!(u16);
value_resolver_for_basetype!(u8);

value_resolver_for_basetype!(isize);
value_resolver_for_basetype!(i64);
value_resolver_for_basetype!(i32);
value_resolver_for_basetype!(i16);
value_resolver_for_basetype!(i8);

value_resolver_for_basetype!(f64);
value_resolver_for_basetype!(f32);
