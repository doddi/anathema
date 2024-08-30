//TODO: Very hacky at the moment to show what should be possible, perhaps need to
// look to see if we can rely on the existing anethema library to provide this
pub(crate) fn get_auto_complete_options(_word: &str) -> Option<Vec<&'static str>> {
    Some(vec!["text", "span", "border", "align", "vstack", "hstack",
    "zstack", "row", "column", "expand", "position", "spacer", "overflow", 
    "padding", "canvas", "container"])
}