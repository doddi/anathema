pub mod contexts;
pub mod error;
mod factory;
pub mod generator;
pub mod layout;
mod style;
mod values;
mod widget;
pub mod views;

// #[cfg(feature = "testing")]
pub mod testing;

pub use generator::Nodes;

pub use crate::factory::{Factory, FactoryContext, WidgetFactory};
pub use crate::layout::{Align, Axis, Direction, LocalPos, Padding, Pos, Region, Display};
pub use crate::style::WidgetStyle;
pub use crate::widget::{AnyWidget, Widget, WidgetContainer};
pub use anathema_render::Color;
