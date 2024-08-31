
pub(crate) fn get_current_word(line: &str, character_pos: u32) -> Option<&str> {
    line.split_whitespace().find(|word| {
        let start = line.find(word).unwrap();
        let end = start + word.len();
        start <= character_pos as usize && character_pos as usize <= end
    })
}
