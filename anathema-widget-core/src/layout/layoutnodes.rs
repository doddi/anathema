use std::ops::{Deref, DerefMut};

use anathema_render::Size;
use anathema_values::Context;

use super::Constraints;
use crate::contexts::LayoutCtx;
use crate::error::Result;
use crate::{Nodes, Padding, WidgetContainer};

pub struct LayoutNodes<'nodes, 'state, 'expr> {
    nodes: &'nodes mut Nodes<'expr>,
    layout: LayoutCtx,
    context: Context<'state, 'expr>,
}

impl<'nodes, 'state, 'expr> LayoutNodes<'nodes, 'state, 'expr> {
    pub fn new(
        nodes: &'nodes mut Nodes<'expr>,
        constraints: Constraints,
        padding: Padding,
        context: Context<'state, 'expr>,
    ) -> Self {
        let layout = LayoutCtx::new(constraints, padding);
        Self {
            nodes,
            layout,
            context,
        }
    }

    pub fn set_constraints(&mut self, constraints: Constraints) {
        self.layout.constraints = constraints;
    }

    pub fn padded_constraints(&self) -> Constraints {
        self.layout.padded_constraints()
    }

    pub fn next<F>(&mut self, mut f: F) -> Result<()>
    where
        F: FnMut(LayoutNode<'_, '_, 'expr>) -> Result<()>
    {
        self.nodes.next(
            &self.context,
            &self.layout,
            &mut |widget, children, context| {
                let node = LayoutNode {
                    widget,
                    children,
                    context,
                };
                f(node)
            },
        )?;

        Ok(())
    }

    pub fn for_each<F>(&mut self, mut f: F)
    where
        F: FnMut(LayoutNode<'_, '_, 'expr>) -> Result<()>
    {
        while let Ok(()) = self.next(&mut f) {}
    }


    pub fn filter<F>(&mut self, f: F) -> impl Iterator<Item = LayoutNode<'_, 'state, 'expr>> + '_
    where
        F: Fn(&WidgetContainer<'expr>) -> bool + 'static
    {
        self.nodes
            .iter_mut()
            .filter(move |(widget, _)| f(*widget))
            .map(|(widget, children)| LayoutNode {
                widget,
                children,
                context: &self.context,
            })
    }
}

pub struct LayoutNode<'widget, 'state, 'expr> {
    widget: &'widget mut WidgetContainer<'expr>,
    children: &'widget mut Nodes<'expr>,
    context: &'widget Context<'state, 'expr>,
}

impl<'widget, 'state, 'expr> LayoutNode<'widget, 'state, 'expr> {
    pub fn layout(&mut self, constraints: Constraints) -> Result<Size> {
        self.widget.layout(self.children, constraints, self.context)
    }
}

impl<'widget, 'state, 'expr> Deref for LayoutNode<'widget, 'state, 'expr> {
    type Target = WidgetContainer<'expr>;

    fn deref(&self) -> &Self::Target {
        self.widget
    }
}

impl<'widget, 'state, 'expr> DerefMut for LayoutNode<'widget, 'state, 'expr> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.widget
    }
}
