use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use tokio::sync::oneshot;

use super::{FocusBackend, FocusError, ForegroundWindow};
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
        let url = WebviewUrl::App(format!("/form/{snippet_id}").into());
        let (x, y) = monitor_center(hwnd, 400, 240);
        let mut builder = WebviewWindowBuilder::new(&self.0, "form", url)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .inner_size(400.0, 240.0);
        if let Some((x, y)) = x.zip(y) {
            builder = builder.position(x as f64, y as f64);
        }
        builder.build().map_err(|error| FormError::Window(error.to_string()))?;
        Ok(())
    }
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

    pub async fn run(
        &self,
        snippet: &Snippet,
        hwnd: ForegroundWindow,
    ) -> Result<FormOutcome, FormError> {
        let (tx, rx) = oneshot::channel();
        {
            let mut in_flight = self.in_flight.lock().unwrap();
            if in_flight.is_some() {
                return Err(FormError::AlreadyOpen);
            }
            self.sink.open_form(&snippet.id, hwnd)?;
            let state = FormState::Pending {
                hwnd,
                snippet_id: Arc::<str>::from(snippet.id.clone()),
            };
            *in_flight = Some(InFlight {
                state,
                tx,
            });
        }

        let result = rx.await.map(|state| match state {
            FormState::Submitted { values } => FormOutcome::Submitted(values),
            FormState::Cancelled => FormOutcome::Cancelled,
            FormState::Pending { .. } => FormOutcome::Cancelled,
        });
        self.in_flight.lock().unwrap().take();
        result.map_err(|_| FormError::ChannelClosed)
    }

    pub fn submit(&self, snippet_id: &str, values: BTreeMap<String, String>) {
        if let Some(in_flight) = self.in_flight.lock().unwrap().take() {
            if in_flight.snippet_id() == snippet_id {
                let _ = in_flight.tx.send(FormState::Submitted { values });
            } else {
                *self.in_flight.lock().unwrap() = Some(in_flight);
            }
        }
    }

    pub fn cancel(&self, snippet_id: &str) {
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
    match outcome {
        FormOutcome::Submitted(_) => focus.restore_foreground(hwnd),
        FormOutcome::Cancelled => Ok(()),
    }
}

#[cfg(windows)]
fn monitor_center(hwnd: ForegroundWindow, width: i32, height: i32) -> (Option<i32>, Option<i32>) {
    use windows::Win32::{
        Foundation::{HWND, RECT},
        Graphics::Gdi::{
            GetMonitorInfoW, MonitorFromWindow, MONITOR_DEFAULTTONEAREST, MONITORINFO,
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
fn monitor_center(_hwnd: ForegroundWindow, _width: i32, _height: i32) -> (Option<i32>, Option<i32>) {
    (None, None)
}
