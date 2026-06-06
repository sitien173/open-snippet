//! Clipboard backup and paste helpers.

use std::{thread, time::Duration};

use crate::expand::ClipboardReader;

use super::{InjectError, KeyboardAction, KeyboardSink};

const CF_TEXT_FORMAT: u32 = 1;
const CF_UNICODETEXT_FORMAT: u32 = 13;

#[derive(Debug, Clone)]
pub struct ClipboardFormat {
    pub format: u32,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ClipboardSnapshot {
    pub formats: Vec<ClipboardFormat>,
}

impl ClipboardSnapshot {
    pub fn text_content(&self) -> Option<String> {
        #[cfg(windows)]
        {
            self.formats
                .iter()
                .find(|entry| entry.format == CF_UNICODETEXT_FORMAT)
                .and_then(|entry| decode_utf16_bytes(&entry.bytes))
        }

        #[cfg(not(windows))]
        {
            None
        }
    }
}

pub trait ClipboardBackend: ClipboardReader + Send + 'static {
    fn paste(
        &mut self,
        sink: &mut dyn KeyboardSink,
        text: &str,
        timeout: Duration,
    ) -> Result<(), InjectError>;
}

pub struct MockClipboardBackend;

impl ClipboardReader for MockClipboardBackend {
    fn read_text(&mut self) -> Option<String> {
        None
    }
}

impl ClipboardBackend for MockClipboardBackend {
    fn paste(
        &mut self,
        sink: &mut dyn KeyboardSink,
        text: &str,
        _timeout: Duration,
    ) -> Result<(), InjectError> {
        // SECURITY: clipboard payload is user content; log redacted text only.
        tracing::debug!(
            text = %crate::log_init::redact::redact_str(
                text,
                crate::log_init::redact::FieldKind::ClipboardText
            ),
            chars = text.chars().count(),
            "mock clipboard paste"
        );
        sink.send(KeyboardAction::Paste(text.to_string()));
        Ok(())
    }
}

pub struct SystemClipboardBackend;

impl ClipboardReader for SystemClipboardBackend {
    fn read_text(&mut self) -> Option<String> {
        #[cfg(windows)]
        {
            let text = capture_clipboard().ok()?.text_content();
            if let Some(text) = &text {
                // SECURITY: clipboard payload is user content; log redacted text only.
                tracing::debug!(
                    text = %crate::log_init::redact::redact_str(
                        text,
                        crate::log_init::redact::FieldKind::ClipboardText
                    ),
                    chars = text.chars().count(),
                    "read clipboard text"
                );
            }
            text
        }

        #[cfg(not(windows))]
        {
            None
        }
    }
}

impl ClipboardBackend for SystemClipboardBackend {
    fn paste(
        &mut self,
        sink: &mut dyn KeyboardSink,
        text: &str,
        timeout: Duration,
    ) -> Result<(), InjectError> {
        #[cfg(windows)]
        {
            // SECURITY: clipboard payload is user content; log redacted text only.
            tracing::debug!(
                text = %crate::log_init::redact::redact_str(
                    text,
                    crate::log_init::redact::FieldKind::ClipboardText
                ),
                chars = text.chars().count(),
                "system clipboard paste"
            );
            let snapshot = capture_clipboard()?;
            let _guard = ClipboardGuard::open(timeout)?;
            clear_clipboard()?;
            set_clipboard_text_internal(text)?;
            sink.send(KeyboardAction::Paste(text.to_string()));
            thread::sleep(Duration::from_millis(10));
            restore_clipboard(&snapshot)?;
            Ok(())
        }

        #[cfg(not(windows))]
        {
            let _ = (sink, text, timeout);
            Err(InjectError::new(
                "clipboard paste unsupported on this platform",
            ))
        }
    }
}

#[derive(Default)]
pub struct TestClipboardBackend {
    text: Option<String>,
}

impl TestClipboardBackend {
    pub fn with_text(text: impl Into<String>) -> Self {
        Self {
            text: Some(text.into()),
        }
    }
}

impl ClipboardReader for TestClipboardBackend {
    fn read_text(&mut self) -> Option<String> {
        self.text.clone()
    }
}

impl ClipboardBackend for TestClipboardBackend {
    fn paste(
        &mut self,
        sink: &mut dyn KeyboardSink,
        text: &str,
        _timeout: Duration,
    ) -> Result<(), InjectError> {
        // SECURITY: clipboard payload is user content; log redacted text only.
        tracing::debug!(
            text = %crate::log_init::redact::redact_str(
                text,
                crate::log_init::redact::FieldKind::ClipboardText
            ),
            chars = text.chars().count(),
            "test clipboard paste"
        );
        sink.send(KeyboardAction::Paste(text.to_string()));
        Ok(())
    }
}

#[cfg(windows)]
struct ClipboardGuard;

#[cfg(windows)]
impl ClipboardGuard {
    fn open(timeout: Duration) -> Result<Self, InjectError> {
        use std::time::Instant;
        use windows::Win32::System::DataExchange::OpenClipboard;

        let started = Instant::now();
        loop {
            let opened = unsafe {
                // SAFETY: opening the process clipboard is required before clipboard API use.
                OpenClipboard(None)
            };
            if opened.is_ok() {
                return Ok(Self);
            }
            if started.elapsed() >= timeout {
                return Err(InjectError::new("timed out acquiring clipboard"));
            }
            thread::sleep(Duration::from_millis(5));
        }
    }
}

#[cfg(windows)]
impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        use windows::Win32::System::DataExchange::CloseClipboard;

        unsafe {
            // SAFETY: closes the process clipboard acquired by ClipboardGuard::open.
            let _ = CloseClipboard();
        }
    }
}

#[cfg(windows)]
pub fn capture_clipboard() -> Result<ClipboardSnapshot, InjectError> {
    use windows::Win32::System::DataExchange::{EnumClipboardFormats, GetClipboardData};

    let _guard = ClipboardGuard::open(Duration::from_millis(50))?;
    let mut formats = Vec::new();
    let mut format = 0u32;

    loop {
        format = unsafe {
            // SAFETY: valid enumeration while clipboard is open.
            EnumClipboardFormats(format)
        };
        if format == 0 {
            break;
        }

        let handle = unsafe {
            // SAFETY: clipboard is open and format was returned by EnumClipboardFormats.
            GetClipboardData(format)
        }
        .map_err(|error| InjectError::new(error.to_string()))?;

        let Some(bytes) = copy_global_bytes(windows::Win32::Foundation::HGLOBAL(handle.0)) else {
            continue;
        };
        formats.push(ClipboardFormat { format, bytes });
    }

    Ok(ClipboardSnapshot { formats })
}

#[cfg(windows)]
pub fn restore_clipboard(snapshot: &ClipboardSnapshot) -> Result<(), InjectError> {
    let _guard = ClipboardGuard::open(Duration::from_millis(50))?;
    clear_clipboard()?;

    for format in &snapshot.formats {
        set_clipboard_bytes(format.format, &format.bytes)?;
    }
    Ok(())
}

#[cfg(windows)]
pub fn set_clipboard_text(text: &str) -> Result<(), InjectError> {
    let _guard = ClipboardGuard::open(Duration::from_millis(50))?;
    clear_clipboard()?;
    set_clipboard_text_internal(text)
}

#[cfg(windows)]
fn set_clipboard_text_internal(text: &str) -> Result<(), InjectError> {
    let mut utf16: Vec<u16> = text.encode_utf16().collect();
    utf16.push(0);
    let mut utf16_bytes = Vec::with_capacity(utf16.len() * 2);
    for unit in utf16 {
        utf16_bytes.extend_from_slice(&unit.to_le_bytes());
    }

    let mut ansi_bytes = text.as_bytes().to_vec();
    ansi_bytes.push(0);

    set_clipboard_bytes(CF_UNICODETEXT_FORMAT, &utf16_bytes)?;
    set_clipboard_bytes(CF_TEXT_FORMAT, &ansi_bytes)?;
    Ok(())
}

#[cfg(windows)]
fn clear_clipboard() -> Result<(), InjectError> {
    use windows::Win32::System::DataExchange::EmptyClipboard;

    unsafe {
        // SAFETY: clipboard is open before clear_clipboard is called.
        EmptyClipboard().map_err(|error| InjectError::new(error.to_string()))
    }
}

#[cfg(windows)]
fn set_clipboard_bytes(format: u32, bytes: &[u8]) -> Result<(), InjectError> {
    use windows::Win32::{
        Foundation::HANDLE,
        System::{
            DataExchange::SetClipboardData,
            Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE},
        },
    };

    unsafe {
        // SAFETY: allocates a movable global buffer owned by the clipboard after SetClipboardData succeeds.
        let memory = GlobalAlloc(GMEM_MOVEABLE, bytes.len())
            .map_err(|error| InjectError::new(error.to_string()))?;
        if memory.is_invalid() {
            return Err(InjectError::new("GlobalAlloc failed"));
        }
        // SAFETY: freshly allocated global memory is valid to lock and initialize.
        let locked = GlobalLock(memory);
        if locked.is_null() {
            return Err(InjectError::new("GlobalLock failed"));
        }
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), locked.cast::<u8>(), bytes.len());
        let _ = GlobalUnlock(memory);
        // SAFETY: clipboard is open and takes ownership of the memory handle on success.
        SetClipboardData(format, HANDLE(memory.0))
            .map_err(|error| InjectError::new(error.to_string()))?;
        Ok(())
    }
}

#[cfg(windows)]
fn copy_global_bytes(handle: windows::Win32::Foundation::HGLOBAL) -> Option<Vec<u8>> {
    use windows::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};

    unsafe {
        // SAFETY: handle came from GetClipboardData for the current open clipboard session.
        let locked = GlobalLock(handle);
        if locked.is_null() {
            return None;
        }
        let size = GlobalSize(handle);
        if size == 0 {
            let _ = GlobalUnlock(handle);
            return None;
        }
        let slice = std::slice::from_raw_parts(locked.cast::<u8>(), size);
        let bytes = slice.to_vec();
        let _ = GlobalUnlock(handle);
        Some(bytes)
    }
}

#[cfg(windows)]
fn decode_utf16_bytes(bytes: &[u8]) -> Option<String> {
    let mut units = Vec::new();
    for chunk in bytes.chunks_exact(2) {
        let unit = u16::from_le_bytes([chunk[0], chunk[1]]);
        if unit == 0 {
            break;
        }
        units.push(unit);
    }
    String::from_utf16(&units).ok()
}
