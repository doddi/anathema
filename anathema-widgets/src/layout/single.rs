use anathema_render::Size;
use anathema_values::Context;
use anathema_widget_core::contexts::LayoutCtx;
use anathema_widget_core::error::{Error, Result};
use anathema_widget_core::generator::Nodes;
use anathema_widget_core::layout::{Constraints, Layout};
use anathema_widget_core::{WidgetContainer, LayoutNodes};

pub struct Single;

impl Layout for Single {
    fn layout<'nodes, 'expr, 'state>(
        &mut self,
        nodes: LayoutNodes<'nodes, 'expr, 'state>,
    ) -> Result<Size> {
        let constraints = nodes.padded_constraints();
        let mut size = Size::ZERO;

        nodes.next(|widget| {
            size = widget.layout(constraints)?;
            Ok(())
        });

        Ok(size)
    }

    // fn layout<'widget, 'parent>(
    //     &mut self,
    //     ctx: &mut LayoutCtx,
    //     children: &mut Nodes,
    //     data: Context<'_, '_>,
    // ) -> Result<()> {
    //     let constraints = ctx.padded_constraints();

    //     if let Some(size) = children.next(data.state, data.scope, ctx).transpose()? {
    //         self.0 = size;
    //         // TODO do we need to deal with insufficient space here?
    //     //     *size = match widget.layout(children, constraints, store) {
    //     //         Ok(s) => s,
    //     //         Err(Error::InsufficientSpaceAvailble) => return Ok(()),
    //     //         err @ Err(_) => err?,
    //     //     };
    //     }

    //     Ok(())
    // }

    // fn finalize(&mut self, nodes: &mut Nodes) -> Size {
    //     self.0
    // }
}
