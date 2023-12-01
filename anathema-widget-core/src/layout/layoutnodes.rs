use std::ops::{Deref, DerefMut};

use anathema_render::Size;
use anathema_values::Context;

use super::Constraints;
use crate::contexts::LayoutCtx;
use crate::error::Result;
use crate::{Nodes, Padding, WidgetContainer};

pub struct LayoutNodes<'nodes, 'expr, 'state> {
    nodes: &'nodes mut Nodes<'expr>,
    layout: LayoutCtx,
    context: Context<'state, 'expr>,
}

impl<'nodes, 'expr, 'state> LayoutNodes<'nodes, 'expr, 'state> {
    pub fn new(
        nodes: &'nodes mut Nodes<'expr>,
        constraints: Constraints,
        context: Context<'state, 'expr>,
    ) -> Self {
        let layout = LayoutCtx::new(constraints, Padding::ZERO);
        Self {
            nodes,
            layout,
            context,
        }
    }

    pub fn next<F>(&mut self, mut f: F)
    where
        F: FnMut(LayoutNode<'_, 'expr, '_>),
    {
        self.nodes.next(
            &self.context,
            &self.layout,
            &mut |widget, children, context| {
                let node = LayoutNode {
                    widget,
                    children,
                    context,
                    constraints: Constraints::ZERO,
                };
                f(node);
                Ok(())
            },
        );
    }

    pub fn layout<F>(&mut self) -> Size
    where
        F: FnMut(LayoutNode<'_, 'expr, '_>),
    {
        let mut size = Size::ZERO;
        self.nodes.next(
            &self.context,
            &self.layout,
            &mut |widget, children, context| {
                let mut node = LayoutNode {
                    widget,
                    children,
                    context,
                    constraints: Constraints::ZERO,
                };
                size = node.layout().unwrap();
                Ok(())
            },
        );

        size
    }

    pub fn for_each<F>(&mut self, mut f: F)
    where
        F: FnMut(LayoutNode<'_, 'expr, '_>),
    {
        loop {
            self.next(&mut f)
        }
    }
}

pub struct LayoutNode<'widget, 'expr, 'state> {
    widget: &'widget mut WidgetContainer<'expr>,
    children: &'widget mut Nodes<'expr>,
    context: &'widget Context<'state, 'expr>,
    constraints: Constraints,
}

impl<'widget, 'expr, 'state> LayoutNode<'widget, 'expr, 'state> {
    pub fn layout(&mut self) -> Result<Size> {
        self.widget
            .layout(self.children, self.constraints, self.context)
    }
}

impl<'widget, 'expr, 'state> Deref for LayoutNode<'widget, 'expr, 'state> {
    type Target = WidgetContainer<'expr>;

    fn deref(&self) -> &Self::Target {
        self.widget
    }
}

impl<'widget, 'expr, 'state> DerefMut for LayoutNode<'widget, 'expr, 'state> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.widget
    }
}
