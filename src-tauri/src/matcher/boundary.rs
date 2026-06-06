//! Boundary classification helpers for trigger matching.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryState {
    StartOfBuffer,
    Whitespace,
    Punctuation,
    Other,
}

pub fn classify_boundary_char(ch: char) -> BoundaryState {
    if ch.is_whitespace() {
        BoundaryState::Whitespace
    } else if ch.is_ascii_punctuation() || is_unicode_punctuation(ch) {
        BoundaryState::Punctuation
    } else {
        BoundaryState::Other
    }
}

fn is_unicode_punctuation(ch: char) -> bool {
    matches!(
        ch,
        '“'
            | '”'
            | '‘'
            | '’'
            | '«'
            | '»'
            | '…'
            | '–'
            | '—'
            | '•'
            | '·'
    )
}
