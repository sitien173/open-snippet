pub trait ClipboardReader {
    fn read_text(&mut self) -> Option<String>;
}

pub fn resolve_clipboard(reader: &mut impl ClipboardReader) -> String {
    let text = reader.read_text().unwrap_or_default();
    // SECURITY: clipboard text is user content and may contain secrets.
    tracing::debug!(
        text = %crate::log_init::redact::redact_str(
            &text,
            crate::log_init::redact::FieldKind::ClipboardText
        ),
        chars = text.chars().count(),
        "resolved clipboard placeholder"
    );
    text
}
