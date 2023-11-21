use anathema_render::Size;
use anathema_values::{
    Attributes, Context, Deferred, DynValue, NodeId, Path, Resolver, State, Value, ValueExpr,
    ValueRef, ValueResolver,
};

pub use self::controlflow::{ElseExpr, IfExpr};
use super::nodes::{IfElse, LoopNode, Single};
use crate::error::Result;
use crate::factory::FactoryContext;
use crate::generator::nodes::{Node, NodeKind, Nodes};
use crate::{Display, Factory, Padding, Pos, WidgetContainer};

mod controlflow;

// -----------------------------------------------------------------------------
//   - A single Node -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct SingleNode {
    pub ident: String,
    pub text: Option<ValueExpr>,
    pub attributes: Attributes,
    pub children: Vec<Expression>,
}

impl SingleNode {
    fn eval<'e>(&'e self, context: &Context<'_, 'e>, node_id: NodeId) -> Result<Node<'e>> {
        // TODO: add > < >= <=, this message is not really about single nodes, but about evaluating
        // values, however this message was attached to another message so here we are... (the
        // other message was an issue that is now resolved under the name of FactoryContext)

        let scope = context.new_scope();

        // TODO: remove this nonsense
        // let mut resolver = Resolver::new(context, Some(&node_id));
        // if let Some(text) = &self.text {
        //     let path = resolver.resolve_path(text);
        //     eprintln!("{:?}", path);
        // }
        // let text = format!("{:?}", self.text);

        let text = self
            .text
            .as_ref()
            .map(|text| String::init_value(context, Some(&node_id), text))
            .unwrap_or_default();

        let context = FactoryContext::new(
            context,
            node_id.clone(),
            &self.ident,
            &self.attributes,
            text,
        );

        let widget = WidgetContainer {
            display: context.get("display"),
            background: context.get("background"),       //context.background(),
            padding: Padding::ZERO, // context.padding(),
            pos: Pos::ZERO,
            size: Size::ZERO,
            node_id: node_id.clone(),
            inner: Factory::exec(context)?,
            expr: None,
        };

        let node = Node {
            kind: NodeKind::Single(Single {
                widget,
                children: Nodes::new(&self.children, node_id.child(0)),
            }),
            node_id,
            scope,
        };

        Ok(node)
    }
}

// -----------------------------------------------------------------------------
//   - Loop -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub(in crate::generator) enum Collection<'e> {
    Static(&'e [ValueExpr]),
    State {
        len: usize,
        path: Path,
        expr: ValueExpr,
    },
    Empty,
}

impl<'e> Collection<'e> {
    pub(super) fn push(&mut self) {
        if let Collection::State { len, .. } = self {
            *len += 1;
        }
    }

    pub(super) fn insert(&mut self, index: usize) {
        if let Collection::State { len, .. } = self {
            if index <= *len {
                *len += 1;
            }
        }
    }

    pub(super) fn remove(&mut self) {
        if let Collection::State { len, .. } = self {
            if *len > 0 {
                *len -= 1;
            }
        }
    }
}

#[derive(Debug)]
pub struct LoopExpr {
    pub body: Vec<Expression>,
    pub binding: Path,
    pub collection: ValueExpr,
}

impl LoopExpr {
    fn eval<'e>(&'e self, context: &Context<'_, 'e>, node_id: NodeId) -> Result<Node<'e>> {
        // Need to know if this is a collection or a path
        let collection = match &self.collection {
            ValueExpr::List(list) => Collection::Static(list),
            col => {
                let mut resolver = Deferred::new(context);
                let val = resolver.resolve(col);
                match val {
                    ValueRef::Expressions(list) => Collection::Static(list),
                    ValueRef::Deferred(path) => {
                        // let before = path.to_string();
                        // let path = resolver.resolve_path(col);
                        // if let Some(path) = &path {
                        //     let after = path.to_string();
                        //     eprintln!("{after}");
                        // }
                        // let len = match path {
                        //     Some(ref path) => match context.state.get(path, Some(&node_id)) {
                        //         ValueRef::List(list) => list.len(),
                        //         _ => 0,
                        //     },
                        //     None => 0,
                        // };

                        let len = match context.state.get(&path, Some(&node_id)) {
                            ValueRef::List(list) => list.len(),
                            _ => 0,
                        };

                        Collection::State {
                            path,
                            len,
                            expr: self.collection.clone(),
                        }
                    }
                    _ => Collection::Empty,
                }
                // // ^^
                // // If this is a list of expressions
                // // then return Collection::Static(val)

                // // let mut resolver = Resolver::new(context, Some(&node_id));

                // // let x = format!("{:?}", context.scopes);

                // let path = resolver.resolve_path(col);
                // let len = match path {
                //     Some(ref path) => match context.state.get(path, Some(&node_id)) {
                //         ValueRef::List(list) => list.len(),
                //         _ => 0,
                //     },
                //     None => 0,
                // };

                // Collection::State {
                //     path,
                //     len,
                //     expr: self.collection.clone(),
                // }
            }
        };

        //let collection = match &self.collection {
        //    ValueExpr::List(expr) => Collection::Static(expr),
        //    value => {
        //        let mut resolver = Resolver::new(context, Some(&node_id));
        //        match resolver.resolve_path(value) {
        //            Some(path) => {
        //                match resolver.lookup_path(&path) {
        //                    // ValueRef::List(col) => Collection::State {
        //                    //     len: col.len(),
        //                    //     path,
        //                    // },
        //                    _ => Collection::Empty,
        //                }
        //            }
        //            None => Collection::Empty,
        //        }
        //    } // Old hat:
        //      //ValueExpr::Ident(_) | ValueExpr::Dot(..) | ValueExpr::Index(..) => {
        //      //    let mut resolver = Resolver::new(context);
        //      //    match self.collection.eval(&mut resolver) {
        //      //        ValueRef::Expressions(expressions) => Collection::ValueExpressions(expressions),
        //      //        ValueRef::List(collection) => Collection::State {
        //      //            len: collection.len(),
        //      //            path,
        //      //        }
        //      //        _ => Collection::Empty,
        //      //    }

        //      //    //
        //      //    match resolver.resolve_path(&self.collection) {
        //      //        Some(path) => match context.resolve_collection(&path, Some(&node_id)) {
        //      //            ValueRef::List(col) => Collection::State {
        //      //                len: col.len(),
        //      //                path,
        //      //            },
        //      //            ValueRef::Deferred(inner_path) => {
        //      //                match context.resolve_collection(&inner_path, Some(&node_id)) {
        //      //                    ValueRef::List(col) => Collection::State {
        //      //                        len: col.len(),
        //      //                        path: inner_path,
        //      //                    },
        //      //                    _ => Collection::Empty,
        //      //                }
        //      //            }
        //      //            _ => Collection::Empty,
        //      //        },
        //      //        None => unreachable!("the deferred resolver should always resolve a path"),
        //      //    }
        //      //}
        //      //_ => Collection::Empty,
        //};

        let loop_node = LoopNode::new(
            &self.body,
            self.binding.clone(),
            collection,
            node_id.child(0),
        );

        let node = Node {
            kind: NodeKind::Loop(loop_node),
            node_id,
            scope: context.new_scope(),
        };

        Ok(node)
    }
}

// -----------------------------------------------------------------------------
//   - Controlflow -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct ControlFlow {
    pub if_expr: IfExpr,
    pub elses: Vec<ElseExpr>,
}

impl ControlFlow {
    fn eval<'e>(&'e self, context: &Context<'_, 'e>, node_id: NodeId) -> Result<Node<'e>> {
        let node = Node {
            kind: NodeKind::ControlFlow(IfElse::new(
                &self.if_expr,
                &self.elses,
                context,
                node_id.child(0),
            )),
            node_id,
            scope: context.new_scope(),
        };
        Ok(node)
    }
}

// -----------------------------------------------------------------------------
//   - Expression -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub enum Expression {
    Node(SingleNode),
    View { ident: ValueExpr, state: Option<ValueExpr> },
    Loop(LoopExpr),
    ControlFlow(ControlFlow),
}

impl Expression {
    pub(crate) fn eval<'a, 'expr>(
        &'expr self,
        context: &Context<'a, 'expr>,
        node_id: NodeId,
    ) -> Result<Node<'expr>> {
        match self {
            Self::Node(node) => node.eval(context, node_id),
            Self::Loop(loop_expr) => loop_expr.eval(context, node_id),
            Self::ControlFlow(controlflow) => controlflow.eval(context, node_id),
            Self::View { ident, state } => panic!("views!"),
        }
    }
}

#[cfg(test)]
mod test {
    use anathema_values::testing::{list, TestState};

    use super::*;
    use crate::contexts::LayoutCtx;
    use crate::generator::testing::*;
    use crate::layout::Constraints;
    use crate::testing::{expression, for_expression};

    impl Expression {
        pub fn test<'a>(self) -> TestExpression<TestState> {
            register_test_widget();

            let constraint = Constraints::new(80, 20);

            TestExpression {
                state: TestState::new(),
                expr: Box::new(self),
                layout: LayoutCtx::new(constraint, Padding::ZERO),
            }
        }
    }

    #[test]
    fn eval_node() {
        let test = expression("test", None, [], []).test();
        let mut node = test.eval().unwrap();
        let (widget, _) = node.single();
        assert_eq!("text", widget.kind());
    }

    #[test]
    fn eval_for() {
        let expr =
            for_expression("item", list([1, 2, 3]), [expression("test", None, [], [])]).test();
        let node = expr.eval().unwrap();
        assert!(matches!(
            node,
            Node {
                kind: NodeKind::Loop { .. },
                ..
            }
        ));
    }
}
