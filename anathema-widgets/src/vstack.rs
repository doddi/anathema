use anathema_render::Size;
use anathema_values::{Attributes, Context, NodeId, ValueExpr, Value};
use anathema_widget_core::contexts::{LayoutCtx, PositionCtx};
use anathema_widget_core::error::Result;
use anathema_widget_core::layout::{Direction, Layouts};
use anathema_widget_core::{
    AnyWidget, FactoryContext, Nodes, Widget, WidgetContainer, WidgetFactory,
};

use crate::layout::vertical::Vertical;

/// A widget that lays out its children vertically.
/// ```text
/// ┌─┐
/// │1│
/// └─┘
/// ┌─┐
/// │2│
/// └─┘
/// ┌─┐
/// │3│
/// └─┘
/// ```
///
/// ```ignore
/// use anathema_widgets::{VStack, Text, Widget, NodeId};
/// let mut vstack = VStack::new(None, None);
/// vstack.children.push(Text::with_text("1").into_container(NodeId::anon()));
/// vstack.children.push(Text::with_text("2").into_container(NodeId::anon()));
/// vstack.children.push(Text::with_text("3").into_container(NodeId::anon()));
/// ```
/// output:
/// ```text
/// 1
/// 2
/// 3
/// ```
#[derive(Debug)]
pub struct VStack {
    /// If a width is provided then the layout constraints will be tight for width
    pub width: Value<usize>,
    /// If a height is provided then the layout constraints will be tight for height
    pub height: Value<usize>,
    /// The minimum width. This will force the minimum constrained width to expand to
    /// this value.
    pub min_width: Value<usize>,
    /// The minimum height. This will force the minimum constrained height to expand to
    /// this value.
    pub min_height: Value<usize>,
}

impl VStack {
    /// Creates a new instance of a `VStack`
    pub fn new(width: impl Into<Value<usize>>, height: impl Into<Value<usize>>) -> Self {
        Self {
            width: width.into(),
            height: height.into(),
            min_width: Value::Empty,
            min_height: Value::Empty,
        }
    }
}

impl Widget for VStack {
    fn kind(&self) -> &'static str {
        "VStack"
    }

    fn layout<'e>(
        &mut self,
        children: &mut Nodes<'e>,
        layout: &LayoutCtx,
        data: &Context<'_, 'e>,
    ) -> Result<Size> {
        let mut layout = *layout;
        if let Some(width) = self.width.value() {
            layout.constraints.max_width = layout.constraints.max_width.min(*width);
        }
        if let Some(height) = self.height.value() {
            layout.constraints.max_height = layout.constraints.max_height.min(*height);
        }
        if let Some(min_width) = self.min_width.value() {
            layout.constraints.min_width = layout.constraints.min_width.max(*min_width);
        }
        if let Some(min_height) = self.min_height.value() {
            layout.constraints.min_height = layout.constraints.min_height.max(*min_height);
        }

        Layouts::new(Vertical::new(Direction::Forward), &layout).layout(children, data)
    }

    fn position<'tpl>(&mut self, children: &mut Nodes, ctx: PositionCtx) {
        let mut pos = ctx.pos;
        for (widget, children) in children.iter_mut() {
            widget.position(children, pos);
            pos.y += widget.outer_size().height as i32;
        }
    }
}

pub(crate) struct VStackFactory;

impl WidgetFactory for VStackFactory {
    fn make(&self, ctx: FactoryContext<'_>) -> Result<Box<dyn AnyWidget>> {
        let width = ctx.get_usize("width");
        let height = ctx.get_usize("height");
        let mut widget = VStack::new(width, height);
        widget.min_width = ctx.get_usize("min-width");
        widget.min_height = ctx.get_usize("min-height");
        Ok(Box::new(widget))
    }
}

#[cfg(test)]
mod test {
    use anathema_widget_core::template::{template, template_text, Template};
    use anathema_widget_core::testing::FakeTerm;

    use super::*;
    use crate::testing::test_widget;

    fn children(count: usize) -> Vec<Template> {
        (0..count)
            .map(|i| template("border", (), vec![template_text(i.to_string())]))
            .collect()
    }

    #[test]
    fn only_vstack() {
        let body = children(3);
        let vstack = VStack::new(None, None);
        test_widget(
            vstack,
            body,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═╗
            ║┌─┐            ║
            ║│0│            ║
            ║└─┘            ║
            ║┌─┐            ║
            ║│1│            ║
            ║└─┘            ║
            ║┌─┐            ║
            ║│2│            ║
            ║└─┘            ║
            ╚═══════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn fixed_height_stack() {
        let body = children(10);
        let vstack = VStack::new(None, 6);
        test_widget(
            vstack,
            body,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═╗
            ║┌─┐            ║
            ║│0│            ║
            ║└─┘            ║
            ║┌─┐            ║
            ║│1│            ║
            ║└─┘            ║
            ║               ║
            ║               ║
            ║               ║
            ╚═══════════════╝
            "#,
            ),
        );
    }
}
