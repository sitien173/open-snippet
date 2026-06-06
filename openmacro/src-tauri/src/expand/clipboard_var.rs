pub trait ClipboardReader {
    fn read_text(&mut self) -> Option<String>;
}

pub fn resolve_clipboard(reader: &mut impl ClipboardReader) -> String {
    reader.read_text().unwrap_or_default()
}
