use std::{
    collections::VecDeque,
    fmt,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

use chrono::Utc;
use serde::Serialize;
use serde_json::{Map, Number, Value};
use tracing::{field, Event, Level, Subscriber};
use tracing_subscriber::{layer::Context, registry::LookupSpan, Layer};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LogEntry {
    pub seq: u64,
    pub ts_unix_ms: i64,
    #[serde(serialize_with = "serialize_level")]
    pub level: Level,
    pub target: String,
    pub message: String,
    pub fields: Value,
    pub span_path: Vec<String>,
}

pub struct RingBuffer {
    capacity: usize,
    next_seq: AtomicU64,
    entries: Mutex<VecDeque<LogEntry>>,
}

impl RingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            next_seq: AtomicU64::new(1),
            entries: Mutex::new(VecDeque::with_capacity(capacity)),
        }
    }

    pub fn push(&self, mut entry: LogEntry) -> u64 {
        let seq = self.next_seq.fetch_add(1, Ordering::Relaxed);
        entry.seq = seq;

        let mut entries = self.entries.lock().unwrap();
        if entries.len() == self.capacity {
            entries.pop_front();
        }
        entries.push_back(entry);
        seq
    }

    pub fn slice_since(&self, since_seq: u64) -> Vec<LogEntry> {
        self.entries
            .lock()
            .unwrap()
            .iter()
            .filter(|entry| entry.seq > since_seq)
            .cloned()
            .collect()
    }

    pub fn len(&self) -> usize {
        self.entries.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub struct RingLayer {
    ring: Arc<RingBuffer>,
}

impl RingLayer {
    pub fn new(ring: Arc<RingBuffer>) -> Self {
        Self { ring }
    }
}

impl<S> Layer<S> for RingLayer
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let metadata = event.metadata();
        let mut visitor = JsonFieldVisitor::default();
        event.record(&mut visitor);

        let entry = LogEntry {
            seq: 0,
            ts_unix_ms: Utc::now().timestamp_millis(),
            level: *metadata.level(),
            target: metadata.target().to_string(),
            message: visitor.message,
            fields: Value::Object(visitor.fields),
            span_path: span_path(event, ctx),
        };
        self.ring.push(entry);
    }
}

#[derive(Default)]
struct JsonFieldVisitor {
    message: String,
    fields: Map<String, Value>,
}

impl JsonFieldVisitor {
    fn record_value(&mut self, field: &field::Field, value: Value) {
        if field.name() == "message" {
            self.message = value
                .as_str()
                .map(str::to_string)
                .unwrap_or_else(|| value.to_string());
            return;
        }
        self.fields.insert(field.name().to_string(), value);
    }
}

impl field::Visit for JsonFieldVisitor {
    fn record_debug(&mut self, field: &field::Field, value: &dyn fmt::Debug) {
        let rendered = format!("{value:?}");
        let value = rendered
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
            .map(str::to_string)
            .unwrap_or(rendered);
        self.record_value(field, Value::String(value));
    }

    fn record_i64(&mut self, field: &field::Field, value: i64) {
        self.record_value(field, Value::Number(value.into()));
    }

    fn record_u64(&mut self, field: &field::Field, value: u64) {
        self.record_value(field, Value::Number(value.into()));
    }

    fn record_bool(&mut self, field: &field::Field, value: bool) {
        self.record_value(field, Value::Bool(value));
    }

    fn record_str(&mut self, field: &field::Field, value: &str) {
        self.record_value(field, Value::String(value.to_string()));
    }

    fn record_f64(&mut self, field: &field::Field, value: f64) {
        let value = Number::from_f64(value)
            .map(Value::Number)
            .unwrap_or(Value::Null);
        self.record_value(field, value);
    }
}

fn span_path<S>(event: &Event<'_>, ctx: Context<'_, S>) -> Vec<String>
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    ctx.event_scope(event)
        .map(|scope| {
            scope
                .from_root()
                .map(|span| span.name().to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn serialize_level<S>(level: &Level, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(level.as_str())
}
