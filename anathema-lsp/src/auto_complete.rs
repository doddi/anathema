use std::fmt::{Display, Formatter};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
pub(crate) enum Component {
    Text,
    Span,
    Border,
    Align,
    VStack,
    HStack,
    ZStack,
    Row,
    Column,
    Expand,
    Position,
    Spacer,
    Overflow,
    Padding,
    Canvas,
    Container
}

impl Display for Component {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Component::Text => write!(f, "text"),
            Component::Span => write!(f, "span"),
            Component::Border => write!(f, "border"),
            Component::Align => write!(f, "align"),
            Component::VStack => write!(f, "vstack"),
            Component::HStack => write!(f, "hstack"),
            Component::ZStack => write!(f, "zstack"),
            Component::Row => write!(f, "row"),
            Component::Column => write!(f, "column"),
            Component::Expand => write!(f, "expand"),
            Component::Position => write!(f, "position"),
            Component::Spacer => write!(f, "spacer"),
            Component::Overflow => write!(f, "overflow"),
            Component::Padding => write!(f, "padding"),
            Component::Canvas => write!(f, "canvas"),
            Component::Container => write!(f, "container"),
        }
    }
}

impl TryFrom<&str> for Component {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "text" => Ok(Component::Text),
            "span" => Ok(Component::Span),
            "border" => Ok(Component::Border),
            "align" => Ok(Component::Align),
            "vstack" => Ok(Component::VStack),
            "hstack" => Ok(Component::HStack),
            "zstack" => Ok(Component::ZStack),
            "row" => Ok(Component::Row),
            "column" => Ok(Component::Column),
            "expand" => Ok(Component::Expand),
            "position" => Ok(Component::Position),
            "spacer" => Ok(Component::Spacer),
            "overflow" => Ok(Component::Overflow),
            "padding" => Ok(Component::Padding),
            "canvas" => Ok(Component::Canvas),
            "container" => Ok(Component::Container),
            _ => Err(())
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
enum BorderAttributes {
    Sides,
    Width,
    Height,
    MinWidth,
    MinHeight,
}

impl Display for BorderAttributes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BorderAttributes::Width => write!(f, "width"),
            BorderAttributes::Height => write!(f, "height"),
            BorderAttributes::MinWidth => write!(f, "min_width"),
            BorderAttributes::MinHeight => write!(f, "min_height"),
            BorderAttributes::Sides => write!(f, "sides"),
        }
    }
}

impl TryFrom<&str> for BorderAttributes {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "width" => Ok(BorderAttributes::Width),
            "height" => Ok(BorderAttributes::Height),
            "min_width" => Ok(BorderAttributes::MinWidth),
            "min_height" => Ok(BorderAttributes::MinHeight),
            _ => Err(())
        }
    }
}

pub(crate) fn get_auto_complete_options(line: &str, word: &str) -> Option<Vec<String>> {
    let tokens = line.split(" ");

    // Is this the first token?
    if tokens.count() == 1 {
        // Find the Components that contain the word at the start
        let collection: Vec<_> = Component::iter()
            .filter(|x| x.to_string().starts_with(word))
            .map(|x| x.to_string()).collect();
        if !collection.is_empty() {
            return Some(collection);
        }
    }
    else {
        // Find the component at the start of the line
        let first_token = line.split(" ").next().unwrap();
        // Check the first_token is a component from the Component enum
        if let Ok(component) = Component::try_from(first_token) {
            // Find the component in the list
            match component {
                Component::Border => {
                    let collection: Vec<_> = BorderAttributes::iter()
                        .filter(|x| x.to_string().starts_with(word))
                        .map(|x| x.to_string()).collect();
                    if !collection.is_empty() {
                        return Some(collection);
                    }
                },
                _ => return None
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_complete_options_first_token() {
        let line = "v";
        let word = "v";
        let result = get_auto_complete_options(line, word);
        assert_eq!(result, Some(vec!["vstack".to_string()]));
    }

    #[test]
    fn auto_complete_options_partial_match() {
        let line = "pa";
        let word = "pa";
        let result = get_auto_complete_options(line, word);
        assert_eq!(result, Some(vec!["padding".to_string()]));
    }

    #[test]
    fn auto_complete_options_no_match() {
        let line = "xyz";
        let word = "xyz";
        let result = get_auto_complete_options(line, word);
        assert_eq!(result, None);
    }

    #[test]
    fn auto_complete_options_multiple_match() {
        let line = "s";
        let word = "s";
        let result = get_auto_complete_options(line, word);
        assert_eq!(result, Some(vec![ "span".to_string(), "spacer".to_string()]));
    }

    #[test]
    fn auto_complete_options_not_first_token() {
        let line = "border [s";
        let word = "s";
        let result = get_auto_complete_options(line, word);
        assert_eq!(result, Some(vec!["sides".to_string()]));
    }

    // fn auto_complete_options_empty_word() {
    //     let line = "vstack";
    //     let word = "";
    //     let result = get_auto_complete_options(line, word);
    //     assert_eq!(result, Some(vec!["vstack"]));
    // }
}