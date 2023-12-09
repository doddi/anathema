use std::io::{stdout, Stdout};
use std::time::Instant;

use anathema_render::{size, Screen, Size};
use anathema_values::state::State;
use anathema_values::{drain_dirty_nodes, Context};
use anathema_widget_core::contexts::{LayoutCtx, PaintCtx};
use anathema_widget_core::error::Result;
use anathema_widget_core::expressions::Expression;
use anathema_widget_core::layout::Constraints;
use anathema_widget_core::nodes::{make_it_so, Node, NodeKind, Nodes};
use anathema_widget_core::views::{AnyView, RegisteredViews, TabIndex, View, ViewFn, Views};
use anathema_widget_core::{LayoutNodes, Padding, Pos};
use anathema_widgets::register_default_widgets;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use events::Event;

// use view::View;
// use self::frame::Frame;
// pub use self::meta::Meta;
use crate::events::{EventProvider, Events};

pub mod events;
// mod frame;
// mod meta;
// mod view;

pub struct Runtime<'e, E, ER, S> {
    pub enable_meta: bool,
    pub enable_mouse: bool,
    // views: V,
    state: S,
    screen: Screen,
    output: Stdout,
    constraints: Constraints,
    nodes: Nodes<'e>,
    events: E,
    event_receiver: ER,
    // meta: Meta,
}

impl<'e, E, ER, S> Drop for Runtime<'e, E, ER, S> {
    fn drop(&mut self) {
        let _ = Screen::show_cursor(&mut self.output);
        let _ = disable_raw_mode();
    }
}

impl<'e, E, ER, S: State> Runtime<'e, E, ER, S>
where
    E: Events<S>,
    ER: EventProvider,
{
    pub fn new(
        expressions: &'e [Expression],
        state: S,
        events: E,
        event_receiver: ER,
    ) -> Result<Self> {
        let nodes = make_it_so(expressions);

        register_default_widgets()?;
        enable_raw_mode()?;
        let mut stdout = stdout();
        Screen::hide_cursor(&mut stdout)?;

        let size: Size = size()?.into();
        let constraints = Constraints::new(Some(size.width), Some(size.height));
        let screen = Screen::new(size);

        let inst = Self {
            output: stdout,
            state,
            screen,
            constraints,
            nodes,
            events,
            event_receiver,
            enable_meta: false,
            enable_mouse: false,
            // meta: Meta::new(size),
        };

        Ok(inst)
    }

    // TODO: move this into views
    fn layout(&mut self) -> Result<()> {
        self.nodes.reset_cache();
        let context = Context::root(&self.state);
        let mut nodes =
            LayoutNodes::new(&mut self.nodes, self.constraints, Padding::ZERO, &context);

        nodes.for_each(|mut node| {
            node.layout(self.constraints)?;
            Ok(())
        })?;

        Ok(())
    }

    fn position(&mut self) {
        for (widget, children) in self.nodes.iter_mut() {
            widget.position(children, Pos::ZERO);
        }
    }

    fn paint(&mut self) {
        for (widget, children) in self.nodes.iter_mut() {
            widget.paint(children, PaintCtx::new(&mut self.screen, None));
        }
    }

    fn changes(&mut self) {
        let dirty_nodes = drain_dirty_nodes();
        if dirty_nodes.is_empty() {
            return;
        }

        let context = Context::root(&self.state);

        for (node_id, change) in dirty_nodes {
            self.nodes.update(node_id.as_slice(), &change, &context);
        }

        // TODO: finish this. Need to figure out a good way to notify that
        //       a values should have a sub removed.
        // for node in removed_nodes() {
        // }
    }

    pub fn run(mut self) -> Result<()> {
        if self.enable_mouse {
            Screen::enable_mouse(&mut self.output)?;
        }

        self.screen.clear_all(&mut self.output)?;

        // self.layout()?;

        // // return Ok(());

        // self.position();
        // self.paint();

        'run: loop {
            while let Some(event) = self.event_receiver.next() {
                let event = self.events.event(event, &mut self.nodes, &mut self.state);

                match event {
                    Event::Resize(width, height) => {
                        let size = Size::from((width, height));
                        self.screen.erase();
                        self.screen.render(&mut self.output)?;
                        self.screen.resize(size);
                        self.screen.clear_all(&mut self.output)?;

                        self.constraints.max_width = size.width;
                        self.constraints.max_height = size.height;

                        // self.meta.size = size;
                    }
                    Event::Blur => (),  //self.meta.focus = false,
                    Event::Focus => (), //self.meta.focus = true,
                    Event::Quit => break 'run Ok(()),
                    _ => {}
                }

                if let Some(id) = TabIndex::current() {
                    if let Some(Node {
                        kind: NodeKind::View(view),
                        ..
                    }) = self.nodes.query().get(&id)
                    {
                        let context = Context::root(&self.state);
                        let state = view.state(&context);
                        // if let view.on_event(event);
                    }
                }
            }

            self.changes();

            // *self.meta.count = self.nodes.count();
            let _total = Instant::now();
            self.layout()?;
            // *self.meta.timings.layout = total.elapsed();

            let _now = Instant::now();
            self.position();
            // *self.meta.timings.position = now.elapsed();

            let _now = Instant::now();
            self.paint();
            // *self.meta.timings.paint = now.elapsed();

            let _now = Instant::now();
            self.screen.render(&mut self.output)?;
            // *self.meta.timings.render = now.elapsed();
            // *self.meta.timings.total = total.elapsed();
            self.screen.erase();

            if self.enable_meta {}
        }
    }

    pub fn register_view<F, T>(&self, id: &str, view: F)
    where
        F: Send + 'static + Fn() -> T,
        T: 'static + View + std::fmt::Debug,
    {
        RegisteredViews::add(id.to_string(), view)
    }
}
