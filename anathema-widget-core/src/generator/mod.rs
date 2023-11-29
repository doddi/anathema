mod expressions;
mod nodes;

#[cfg(any(attribute = "testing", test))]
mod testing;

pub use expressions::{ControlFlow, ElseExpr, Expression, IfExpr, LoopExpr, SingleNode, ViewExpr};
pub use nodes::Nodes;

pub fn make_it_so(expressions: &[Expression]) -> Nodes<'_> {
    Nodes::new(expressions, 0.into())
}
