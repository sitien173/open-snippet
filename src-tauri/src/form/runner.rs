use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use tokio::sync::oneshot;

use super::{FocusBackend, FocusError, ForegroundWindow};
use crate::log_init::redact::{redact_str, FieldKind};
use crate::store::Snippet;
use tauri::{AppHandle, Runtime, WebviewUrl, WebviewWindowBuilder};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormState {
    Pending {
        hwnd: ForegroundWindow,
        snippet_id: Arc<str>,
    },
    Submitted {
        values: BTreeMap<String, String>,
    },
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormOutcome {
    Submitted(BTreeMap<String, String>),
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormError {
    AlreadyOpen,
    Window(String),
    ChannelClosed,
}

pub trait WindowSink: Send + Sync + 'static {
    fn open_form(&self, snippet_id: &str, hwnd: ForegroundWindow) -> Result<(), FormError>;
}

#[derive(Clone)]
pub struct AppWindowSink<R: Runtime>(AppHandle<R>);

impl<R: Runtime> AppWindowSink<R> {
    pub fn new(app: AppHandle<R>) -> Self {
        Self(app)
    }
}

impl<R: Runtime> WindowSink for AppWindowSink<R> {
    fn open_form(&self, snippet_id: &str, hwnd: ForegroundWindow) -> Result<(), FormError> {
        let url = WebviewUrl::App(form_route(snippet_id).into());
        let (x, y) = monitor_center(hwnd, 400, 240);
        let mut builder = WebviewWindowBuilder::new(&self.0, "form", url)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .inner_size(400.0, 240.0);
        if let Some((x, y)) = x.zip(y) {
            builder = builder.position(x as f64, y as f64);
        }
        builder
            .build()
            .map_err(|error| FormError::Window(error.to_string()))?;
        Ok(())
    }
}

fn form_route(snippet_id: &str) -> String {
    format!("/form/{}", encode_path_segment(snippet_id))
}

fn encode_path_segment(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                use std::fmt::Write as _;
                let _ = write!(&mut encoded, "%{byte:02X}");
            }
        }
    }
    encoded
}

#[derive(Default)]
pub struct NoopWindowSink;

impl WindowSink for NoopWindowSink {
    fn open_form(&self, _snippet_id: &str, _hwnd: ForegroundWindow) -> Result<(), FormError> {
        Ok(())
    }
}

struct InFlight {
    state: FormState,
    tx: oneshot::Sender<FormState>,
}

impl InFlight {
    fn snippet_id(&self) -> &str {
        match &self.state {
            FormState::Pending { snippet_id, .. } => snippet_id,
            FormState::Submitted { .. } | FormState::Cancelled => "",
        }
    }
}

pub struct FormRunner {
    sink: Arc<dyn WindowSink>,
    in_flight: Mutex<Option<InFlight>>,
}

impl FormRunner {
    pub fn new<R: Runtime>(app: AppHandle<R>) -> Self {
        Self::new_with_sink(AppWindowSink::new(app))
    }

    pub fn new_with_sink(sink: impl WindowSink) -> Self {
        Self {
            sink: Arc::new(sink),
            in_flight: Mutex::new(None),
        }
    }

    #[tracing::instrument(skip(self, snippet), fields(snippet_id = %snippet.id, hwnd = hwnd.0))]
    pub async fn run(
        &self,
        snippet: &Snippet,
        hwnd: ForegroundWindow,
    ) -> Result<FormOutcome, FormError> {
        tracing::info!(snippet_id = %snippet.id, hwnd = hwnd.0, "opening form");
        let (tx, rx) = oneshot::channel();
        {
            let mut in_flight = self.in_flight.lock().unwrap();
            if in_flight.is_some() {
                tracing::warn!(snippet_id = %snippet.id, "form already open");
                return Err(FormError::AlreadyOpen);
            }
            self.sink.open_form(&snippet.id, hwnd)?;
            let state = FormState::Pending {
                hwnd,
                snippet_id: Arc::<str>::from(snippet.id.clone()),
            };
            *in_flight = Some(InFlight { state, tx });
        }

        let result = rx.await.map(|state| match state {
            FormState::Submitted { values } => FormOutcome::Submitted(values),
            FormState::Cancelled => FormOutcome::Cancelled,
            FormState::Pending { .. } => FormOutcome::Cancelled,
        });
        self.in_flight.lock().unwrap().take();
        if let Ok(outcome) = &result {
            tracing::info!(outcome = form_outcome_label(outcome), "form completed");
        }
        result.map_err(|_| FormError::ChannelClosed)
    }

    #[tracing::instrument(skip(self, values), fields(snippet_id = %snippet_id, value_count = values.len()))]
    pub fn submit(&self, snippet_id: &str, values: BTreeMap<String, String>) {
        // SECURITY: form values are user content and may contain secrets.
        let redacted_values = values
            .iter()
            .map(|(key, value)| (key.as_str(), redact_str(value, FieldKind::FormValue)))
            .collect::<BTreeMap<_, _>>();
        tracing::debug!(
            snippet_id,
            value_count = values.len(),
            values = ?redacted_values,
            "submitting form"
        );
        if let Some(in_flight) = self.in_flight.lock().unwrap().take() {
            if in_flight.snippet_id() == snippet_id {
                let _ = in_flight.tx.send(FormState::Submitted { values });
            } else {
                *self.in_flight.lock().unwrap() = Some(in_flight);
            }
        }
    }

    #[tracing::instrument(skip(self), fields(snippet_id = %snippet_id))]
    pub fn cancel(&self, snippet_id: &str) {
        tracing::debug!(snippet_id, "cancelling form");
        if let Some(in_flight) = self.in_flight.lock().unwrap().take() {
            if in_flight.snippet_id() == snippet_id {
                let _ = in_flight.tx.send(FormState::Cancelled);
            } else {
                *self.in_flight.lock().unwrap() = Some(in_flight);
            }
        }
    }
}

pub fn restore_on_submit(
    focus: &(impl FocusBackend + ?Sized),
    hwnd: ForegroundWindow,
    outcome: &FormOutcome,
) -> Result<(), FocusError> {
    tracing::debug!(
        hwnd = hwnd.0,
        outcome = form_outcome_label(outcome),
        "restoring foreground after form"
    );
    match outcome {
        FormOutcome::Submitted(_) => focus.restore_foreground(hwnd),
        FormOutcome::Cancelled => Ok(()),
    }
}

fn form_outcome_label(outcome: &FormOutcome) -> &'static str {
    match outcome {
        FormOutcome::Submitted(_) => "submitted",
        FormOutcome::Cancelled => "cancelled",
    }
}

#[cfg(windows)]
fn monitor_center(hwnd: ForegroundWindow, width: i32, height: i32) -> (Option<i32>, Option<i32>) {
    use windows::Win32::{
        Foundation::{HWND, RECT},
        Graphics::Gdi::{
            GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
        },
    };

    let target = HWND(hwnd.0 as *mut core::ffi::c_void);
    let monitor = unsafe {
        // SAFETY: target HWND is an opaque OS handle used only for monitor lookup.
        MonitorFromWindow(target, MONITOR_DEFAULTTONEAREST)
    };
    if monitor.0.is_null() {
        return (None, None);
    }

    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        rcMonitor: RECT::default(),
        rcWork: RECT::default(),
        dwFlags: 0,
    };
    let got_info = unsafe {
        // SAFETY: info points to valid writable memory and cbSize is initialized.
        GetMonitorInfoW(monitor, &mut info as *mut MONITORINFO as *mut _)
    };
    if !got_info.as_bool() {
        return (None, None);
    }

    let work = info.rcWork;
    let x = work.left + ((work.right - work.left - width) / 2);
    let y = work.top + ((work.bottom - work.top - height) / 2);
    (Some(x), Some(y))
}

#[cfg(not(windows))]
fn monitor_center(
    _hwnd: ForegroundWindow,
    _width: i32,
    _height: i32,
) -> (Option<i32>, Option<i32>) {
    (None, None)
}

#[cfg(test)]
mod tests {
    #[test]
    fn form_route_percent_encodes_snippet_ids() {
        assert_eq!(
            super::form_route("folder/snippet:hi snowman \u{2603}"),
            "/form/folder%2Fsnippet%3Ahi%20snowman%20%E2%98%83"
        );
    }
}
