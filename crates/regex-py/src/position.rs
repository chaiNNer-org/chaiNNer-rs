pub fn to_byte_pos(text: &str, char_pos: usize) -> usize {
    if char_pos == 0 {
        return 0;
    }
    if char_pos > text.len() {
        // the position is out of bounds anyway
        return char_pos;
    }

    if let Some(char_pos) = text.char_indices().nth(char_pos).map(|(i, _)| i) {
        char_pos
    } else {
        // the position is out of bounds
        char_pos + text.chars().count() - text.len()
    }
}

/// A helper struct for translating byte positions to Unicode character positions.
pub struct PosTranslator<'a> {
    text: &'a str,
    known: Vec<(usize, usize)>,
}
impl<'a> PosTranslator<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            known: vec![],
        }
    }

    pub fn get_char_pos(&mut self, byte_pos: usize) -> usize {
        let mut start_offset = (0, 0);
        for k in self.known.iter().rev() {
            if k.0 <= byte_pos {
                start_offset = *k;
                break;
            }
        }

        if start_offset.0 == byte_pos {
            return start_offset.1;
        }

        let text = &self.text[start_offset.0..];

        let mut char_pos = start_offset.1;
        for (i, _) in text.char_indices() {
            if i + start_offset.0 >= byte_pos {
                break;
            }
            char_pos += 1;
        }

        if self.known.last().map_or(true, |l| l.0 < byte_pos) {
            // keep the array sorted
            self.known.push((byte_pos, char_pos));
        }

        char_pos
    }
}
